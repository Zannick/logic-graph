use crate::context::{ContextWrapper, Ctx, HistoryAlias, Wrapper};
use crate::schema::db_states::dsl::*;
use crate::scoring::{BestTimes, EstimatorWrapper, ScoreMetric};
use crate::storage::{get_obj_from_data, serialize_data, serialize_state, NextSteps};
use crate::world::{Location, World};
use diesel::dsl::{min, not, DuplicatedKeys};
use diesel::expression::functions::*;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sql_types::*;
use dotenvy::dotenv;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::env;
use std::marker::PhantomData;

const INVALID_ESTIMATE: u32 = crate::estimates::UNREASONABLE_TIME + 3;
const TEST_DATABASE_URL: &'static str = "mysql://logic_graph@localhost/logic_graph__unittest";

define_sql_function!(
    #[sql_name = "IF"]
    fn sqlif<T: SingleValue>(cond: Bool, case_true: T, case_false: T) -> T
);
define_sql_function! (
    #[sql_name = "VALUES"]
    fn insertvalues<T: SingleValue>(v: T) -> T
);

pub fn env_database_url() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Debug, Default, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::db_states, check_for_backend(Mysql))]
pub struct DBState {
    pub raw_state: Vec<u8>,
    pub progress: u32,
    pub elapsed: u32,
    pub time_since_visit: u32,
    pub estimated_remaining: u32,
    pub step_time: u32,
    pub processed: bool,
    pub queued: bool,
    pub won: bool,
    pub hist: Option<Vec<u8>>,
    pub prev: Option<Vec<u8>>,
    pub next_steps: Option<Vec<u8>>,
}

impl DBState {
    pub fn from_ctx<'w, W, T>(
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
        metric: &impl EstimatorWrapper<'w, W>,
    ) -> DBState
    where
        W: World + 'w,
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
    {
        Self::from_ctx_with_raw(
            serialize_state(ctx.get()),
            ctx,
            is_solution,
            serialized_prev,
            queue,
            metric.estimated_remaining_time(ctx.get()),
        )
    }

    pub fn from_ctx_with_raw<W, T>(
        raw: Vec<u8>,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
        estimated_time_remaining: u32,
    ) -> DBState
    where
        W: World,
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
    {
        let h = ctx.recent_history();
        DBState {
            raw_state: raw,
            progress: ctx.get().progress(),
            elapsed: ctx.elapsed(),
            time_since_visit: ctx.time_since_visit(),
            estimated_remaining: estimated_time_remaining,
            step_time: ctx.recent_dur(),
            won: is_solution,
            hist: if h.is_empty() {
                None
            } else {
                assert!(
                    h.len() == 1,
                    "States encoded in DB must have at most one hist entry, got: {} ({:?})",
                    h.len(),
                    h
                );
                Some(serialize_data(h[0]))
            },
            prev: serialized_prev,
            queued: !is_solution && queue,
            ..Default::default()
        }
    }
}

// Importing all of db_states::dsl prevents naming things the same, so we define these
// in submod to not need to make different names
mod q {
    use crate::context::{Ctx, HistoryAlias};
    use crate::storage::get_obj_from_data;
    use diesel::mysql::Mysql;
    use diesel::prelude::*;
    use diesel::sql_types::*;

    #[derive(Debug, Eq, PartialEq)]
    pub struct DBEntry<T>
    where
        T: Ctx,
    {
        pub state: T,
        pub progress: u32,
        pub elapsed: u32,
        pub time_since_visit: u32,
        pub estimated_remaining: u32,
        pub step_time: u32,
        pub processed: bool,
        pub queued: bool,
        pub won: bool,
        pub hist: Option<HistoryAlias<T>>,
        pub prev: Option<T>,
        pub next_steps: Option<Vec<HistoryAlias<T>>>,
    }

    impl<T> From<super::DBState> for DBEntry<T>
    where
        T: Ctx,
    {
        fn from(value: super::DBState) -> Self {
            Self {
                state: get_obj_from_data(&value.raw_state).unwrap(),
                progress: value.progress,
                elapsed: value.elapsed,
                time_since_visit: value.time_since_visit,
                estimated_remaining: value.estimated_remaining,
                step_time: value.step_time,
                processed: value.processed,
                queued: value.queued,
                won: value.won,
                prev: value.prev.map(|buf| get_obj_from_data(&buf).unwrap()),
                hist: value.hist.map(|buf| get_obj_from_data(&buf).unwrap()),
                next_steps: value.next_steps.map(|buf| get_obj_from_data(&buf).unwrap()),
            }
        }
    }

    #[derive(QueryableByName)]
    #[diesel(check_for_backend(Mysql))]
    pub struct DownstreamState {
        #[diesel(sql_type = Blob)]
        pub raw_state: Vec<u8>,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub old_elapsed: u32,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub new_elapsed: u32,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub new_time_since_visit: u32,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub step_time: u32,
    }

    #[derive(Debug)]
    pub struct DownstreamEntry<T> {
        pub state: T,
        pub old_elapsed: u32,
        pub new_elapsed: u32,
        pub new_time_since_visit: u32,
        pub step_time: u32,
    }

    impl<T> From<DownstreamState> for DownstreamEntry<T>
    where
        T: Ctx,
    {
        fn from(value: DownstreamState) -> Self {
            Self {
                state: get_obj_from_data(&value.raw_state).unwrap(),
                old_elapsed: value.old_elapsed,
                new_elapsed: value.new_elapsed,
                new_time_since_visit: value.new_time_since_visit,
                step_time: value.step_time,
            }
        }
    }

    #[derive(QueryableByName)]
    #[diesel(check_for_backend(Mysql))]
    pub struct HistoryState {
        #[diesel(sql_type = Blob)]
        pub raw_state: Vec<u8>,
        #[diesel(sql_type = Nullable<Blob>)]
        pub prev: Option<Vec<u8>>,
        #[diesel(sql_type = Nullable<Blob>)]
        pub hist: Option<Vec<u8>>,
        #[diesel(sql_type = Unsigned<Integer>)]
        pub elapsed: u32,
    }

    #[derive(Debug)]
    pub struct HistoryEntry<T>
    where
        T: Ctx,
    {
        pub state: T,
        pub prev: Option<T>,
        pub hist: Option<HistoryAlias<T>>,
        pub elapsed: u32,
    }

    impl<T> From<HistoryState> for HistoryEntry<T>
    where
        T: Ctx,
    {
        fn from(value: HistoryState) -> Self {
            Self {
                state: get_obj_from_data(&value.raw_state).unwrap(),
                prev: value.prev.map(|buf| get_obj_from_data(&buf).unwrap()),
                hist: value.hist.map(|buf| get_obj_from_data(&buf).unwrap()),
                elapsed: value.elapsed,
            }
        }
    }
}
pub use q::*;

pub type MysqlPoolConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub type StickyConnection = Option<MysqlPoolConnection>;

pub struct MySQLDB<'w, W, T, const KS: usize, SM> {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    metric: SM,
    phantom: PhantomData<&'w (W, T)>,
}

impl<'w, W, T, const KS: usize, SM> MySQLDB<'w, W, T, KS, SM>
where
    W: World + 'w,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    SM: ScoreMetric<'w, W, T, KS>,
{
    pub fn connect(metric: SM) -> Self {
        let manager = ConnectionManager::new(env_database_url());
        Self {
            pool: Pool::builder()
                .max_size(rayon::current_num_threads() as u32)
                .build(manager)
                .expect("Could not build MySQL connection pool"),
            metric,
            phantom: PhantomData::default(),
        }
    }

    /// Opens a DB connection and starts a test transaction to it, ensuring that no changes are made during the test.
    pub fn with_test_connection(metric: SM) -> Self {
        use diesel_migrations::*;
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let manager = ConnectionManager::new(TEST_DATABASE_URL);
        let db = Self {
            pool: Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("Could not build MySQL connection pool"),
            metric,
            phantom: PhantomData::default(),
        };

        let mut conn = db.pool.get().unwrap();
        // dropping the table will fail if it doesn't exist
        let _ = conn.revert_all_migrations(MIGRATIONS);
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn.begin_test_transaction().unwrap();
        db
    }

    /// Returns an object that will lazily get a pool connection and hold onto it until dropped.
    ///
    /// Most functions that interact with the DB will take a &mut StickyConnection so the caller
    /// can use the same connection across multiple calls. Otherwise provide `&mut StickyConnection::None`
    /// or `&mut db.get_sticky_connection()` to get a pool connection that will be returned after the call.
    pub fn get_sticky_connection(&self) -> StickyConnection {
        StickyConnection::None
    }

    /// Initializes the StickyConnection if necessary and returns the pool connection inside it.
    pub fn sticky<'a>(&self, conn: &'a mut StickyConnection) -> &'a mut MysqlPoolConnection {
        conn.get_or_insert_with(|| self.pool.get().expect("Failed to get a pool connection"))
    }

    pub fn metric(&self) -> &SM {
        &self.metric
    }

    pub fn encode_one_for_upsert(
        &self,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
        conn: &mut StickyConnection,
    ) -> DBState {
        let key = serialize_state(ctx.get());
        let est = if self.exists_raw(&key, conn).unwrap() {
            INVALID_ESTIMATE // shouldn't be written to the db
        } else {
            self.metric.estimated_remaining_time(ctx.get())
        };
        DBState::from_ctx_with_raw(key, ctx, is_solution, serialized_prev, queue, est)
    }

    pub fn encode_many_for_upsert(
        &self,
        ctxs_prevs: Vec<(&ContextWrapper<T>, Option<Vec<u8>>)>,
        world: &W,
        queue: bool,
        conn: &mut StickyConnection,
    ) -> Vec<DBState> {
        let keys = ctxs_prevs
            .iter()
            .map(|(ctx, _)| serialize_state(ctx.get()))
            .collect::<Vec<_>>();
        let emap = FxHashMap::from_iter(self.get_estimates(&keys, conn).unwrap());
        keys.into_par_iter()
            .zip(ctxs_prevs)
            .map(|(key, (ctx, sprev))| {
                let est = emap
                    .get(&key)
                    .copied()
                    .unwrap_or_else(|| self.metric.estimated_remaining_time(ctx.get()));
                DBState::from_ctx_with_raw(key, ctx, world.won(ctx.get()), sprev, queue, est)
            })
            .collect()
    }

    pub fn insert_one(
        &self,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
        conn: &mut StickyConnection,
    ) -> QueryResult<usize> {
        let value = self.encode_one_for_upsert(ctx, is_solution, serialized_prev, queue, conn);
        let new_elapsed = value.elapsed;
        diesel::insert_into(db_states)
            .values(value)
            .on_conflict(DuplicatedKeys)
            .do_update()
            .set((
                time_since_visit.eq(sqlif(
                    elapsed.gt(new_elapsed),
                    insertvalues(time_since_visit),
                    time_since_visit,
                )),
                step_time.eq(sqlif(
                    elapsed.gt(new_elapsed),
                    insertvalues(step_time),
                    step_time,
                )),
                hist.eq(sqlif(elapsed.gt(new_elapsed), insertvalues(hist), hist)),
                prev.eq(sqlif(elapsed.gt(new_elapsed), insertvalues(prev), prev)),
                // must be last as updated values are visible in later changes
                elapsed.eq(sqlif(elapsed.gt(new_elapsed), new_elapsed, elapsed)),
                // If the state was already processed, then it will not be queued.
                // Otherwise, we set it to what we have this time.
                queued.eq(not(processed).and(insertvalues(queued))),
            ))
            .execute(self.sticky(conn))
    }

    pub fn insert_batch(
        &self,
        values: &Vec<DBState>,
        conn: &mut StickyConnection,
    ) -> QueryResult<usize> {
        diesel::insert_into(db_states)
            .values(values)
            .on_conflict(DuplicatedKeys)
            .do_update()
            .set((
                // Overwrite elapsed, time_since, hist, and prev if elapsed is better.
                time_since_visit.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(time_since_visit),
                    time_since_visit,
                )),
                step_time.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(step_time),
                    step_time,
                )),
                hist.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(hist),
                    hist,
                )),
                prev.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(prev),
                    prev,
                )),
                // must be last as updated values are visible in later changes
                elapsed.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(elapsed),
                    elapsed,
                )),
                // If the state was already processed, then it will not be queued.
                // Otherwise, we set it to what we have this time.
                queued.eq(not(processed).and(insertvalues(queued))),
                // Other fields will not change with a better path:
                // progress, estimated_remaining, won, next_steps
            ))
            .execute(self.sticky(conn))
    }

    /// Insert/update both a processed state and its subsequent states.
    /// Assumes the processed state has already been committed.
    /// It's assumed that all subsequent states will be queued if new.
    pub fn insert_processed(
        &self,
        ctx: &ContextWrapper<T>,
        world: &W,
        next_states: &Vec<ContextWrapper<T>>,
        conn: &mut StickyConnection,
    ) -> QueryResult<(Vec<DBState>, usize)> {
        let parent_state = serialize_state(ctx.get());
        let mut next_hists = Vec::new();
        for next_state in next_states {
            let h = next_state.recent_history();
            assert!(
                h.len() == 1,
                "Next states encoded in DB must have exactly one hist entry, got: {} ({:?})",
                h.len(),
                h
            );
            next_hists.push(h[0]);
        }
        let values = self.encode_many_for_upsert(
            next_states
                .iter()
                .zip(std::iter::repeat(Some(&parent_state).cloned()))
                .collect(),
            world,
            true,
            conn,
        );

        let inserts = self.insert_batch(&values, conn)?;

        // Update in a separate command so we don't need to add conditions on these sets for the above
        // insert-on-duplicate-keys-update entries which will never need to update these fields.
        let updates = diesel::update(db_states.filter(raw_state.eq(parent_state)))
            .set((
                processed.eq(true),
                next_steps.eq(serialize_data(next_hists)),
            ))
            .execute(self.sticky(conn))?;
        Ok((values, inserts + updates))
    }

    pub fn insert_processed_and_improve(
        &self,
        ctx: &ContextWrapper<T>,
        world: &W,
        next_states: &Vec<ContextWrapper<T>>,
    ) -> QueryResult<usize> {
        let mut conn = self.get_sticky_connection();
        let (values, count) = self.insert_processed(ctx, world, next_states, &mut conn)?;
        let mut q = queries::improve_downstream(values.len()).into_boxed();
        for value in values {
            q = q.bind::<Blob, _>(value.raw_state);
        }
        Ok(count + q.execute(self.sticky(&mut conn))?)
    }

    pub fn get_record(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<DBEntry<T>> {
        queries::lookup_state(&serialize_state(state))
            .first(self.sticky(conn))
            .map(|row: DBState| row.into())
    }

    pub fn get_best_times(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<BestTimes> {
        queries::get_best_times(&serialize_state(state)).first(self.sticky(conn))
    }

    pub fn get_estimates(
        &self,
        keys: &[Vec<u8>],
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<(Vec<u8>, u32)>> {
        queries::get_estimates(keys).load(self.sticky(conn))
    }

    pub fn get_prev_hist(
        &self,
        state: &T,
        conn: &mut StickyConnection,
    ) -> QueryResult<(Option<T>, Option<HistoryAlias<T>>)> {
        queries::get_prev_hist(&serialize_state(state))
            .first(self.sticky(conn))
            .map(|(p, h): (Option<Vec<u8>>, Option<Vec<u8>>)| {
                (
                    p.map(|buf| get_obj_from_data(&buf).unwrap()),
                    h.map(|buf| get_obj_from_data(&buf).unwrap()),
                )
            })
    }

    pub fn get_next_steps(
        &self,
        state: &T,
        conn: &mut StickyConnection,
    ) -> QueryResult<Option<NextSteps<T>>> {
        queries::get_next_steps(&serialize_state(state))
            .first(self.sticky(conn))
            .map(|data: Option<Vec<u8>>| {
                if let Some(buf) = &data {
                    Some(get_obj_from_data::<NextSteps<T>>(buf).unwrap())
                } else {
                    None
                }
            })
    }

    pub fn has_next_steps(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<bool> {
        queries::has_next_steps(&serialize_state(state)).first(self.sticky(conn))
    }

    pub fn exists(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<bool> {
        self.exists_raw(&serialize_state(state), conn)
    }

    pub fn exists_raw(&self, key: &Vec<u8>, conn: &mut StickyConnection) -> QueryResult<bool> {
        queries::row_exists(key).first(self.sticky(conn))
    }

    pub fn full_history(
        &self,
        state: &T,
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<HistoryEntry<T>>> {
        self.full_history_raw(&serialize_state(state), conn)
    }

    pub fn full_history_raw(
        &self,
        key: &[u8],
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<HistoryEntry<T>>> {
        queries::full_history()
            .bind::<Blob, _>(key)
            .load(self.sticky(conn))
            .map(|vec| vec.into_iter().map(|hs: HistoryState| hs.into()).collect())
    }

    /// Recursively update the times for the states downstream from the given states.
    ///
    /// This is guaranteed safe and correct if no given state that had an improvement to pass
    /// downstream is in another given state's downstream, which should automatically be the case as long as
    /// as the given states passed in together are from the same "next" cohort of a processed state.
    pub fn improve_downstream(
        &self,
        states: &[&T],
        conn: &mut StickyConnection,
    ) -> QueryResult<usize> {
        let mut q = queries::improve_downstream(states.len()).into_boxed();
        for state in states {
            q = q.bind::<Blob, _>(serialize_state(*state))
        }
        q.execute(self.sticky(conn))
    }

    pub fn test_downstream(
        &self,
        states: &[&T],
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<DownstreamEntry<T>>> {
        let mut q = queries::test_downstream(states.len()).into_boxed();
        for state in states {
            q = q.bind::<Blob, _>(serialize_state(*state));
        }
        q.load(self.sticky(conn)).map(|vec| {
            vec.into_iter()
                .map(|ds: DownstreamState| ds.into())
                .collect()
        })
    }
}

#[allow(unused)]
mod queries {
    use crate::models::DBState;
    use crate::schema::db_states::dsl::*;
    use crate::scoring::BestTimes;
    use diesel::dsl::{auto_type, exists, select, AsSelect};
    use diesel::expression::UncheckedBind;
    use diesel::mysql::Mysql;
    use diesel::prelude::*;
    use diesel::query_builder::{QueryFragment, SqlQuery};
    use diesel::sql_types::*;

    define_sql_function!(
        #[sql_name = "IF"]
        fn sqlif<T: SingleValue>(cond: Bool, case_true: T, case_false: T) -> T
    );
    define_sql_function! (
        #[sql_name = "VALUES"]
        fn insertvalues<T: SingleValue>(v: T) -> T
    );

    #[auto_type(type_case = "PascalCase")]
    pub fn lookup_state<'a>(key: &'a [u8]) -> _ {
        db_states.find(key)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_best_times<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select::<AsSelect<BestTimes, Mysql>>(BestTimes::as_select())
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_next_steps<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(next_steps)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn has_next_steps<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(next_steps.is_not_null())
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_processed<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(processed)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_prev_hist<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select((prev, hist))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_elapsed_prev_hist<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select((elapsed, prev, hist))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_estimate<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(estimated_remaining)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_estimates<'a>(keys: &'a [Vec<u8>]) -> _ {
        db_states
            .filter(raw_state.eq_any(keys))
            .select((raw_state, estimated_remaining))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn row_exists<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        select(exists(row.select(1i32.into_sql::<Integer>())))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn is_queued() -> _ {
        queued.eq(true)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn unprocessed() -> _ {
        processed.eq(false)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn preserved() -> _ {
        let q: IsQueued = is_queued();
        let u: Unprocessed = unprocessed();
        q.and(u)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn available(max_time: u32) -> _ {
        let p: Preserved = preserved();
        p.and(elapsed.lt(max_time))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn score() -> _ {
        elapsed + estimated_remaining
    }

    pub fn full_history() -> SqlQuery {
        // If the first result isn't the starting state, there's a recursive loop in the states
        // that needs to be fixed.
        diesel::sql_query(
            r#"
            WITH RECURSIVE FullHistory(raw_state, prev, hist, elapsed)
            AS (
            -- Anchor definition = the end state
            SELECT raw_state, prev, hist, elapsed
                FROM db_states
                WHERE raw_state = ?
            UNION DISTINCT
            -- Recursive definition = the state pointed to by the previous state's prev
            SELECT db.raw_state, db.prev, db.hist, db.elapsed
            FROM db_states as db
            INNER JOIN FullHistory as fh
                ON db.raw_state = fh.prev
            )
            SELECT raw_state, prev, hist, elapsed
            FROM FullHistory
            ORDER BY elapsed
            "#,
        )
    }

    fn downstream(n: usize) -> SqlQuery {
        // We can support performing downstream updates from multiple states
        // as long as they are not in each other's downstreams
        // because each is the root of its own subtree, so they cannot share any downstream states
        // without being fully contained in another subtree.
        // As long as the states are from the same "next" cohort of a processed state, then if one of them
        // is updated, it's guaranteed to be a root directly under that processed state. And if one is not
        // updated, then it will not generate changes in the recursive part of the query thanks to the WHERE condition
        // even if it is in another downstream.
        // And we leave mysql in autocommit mode to ensure that all statements are transactions
        diesel::sql_query(format!(
            r#"
            WITH RECURSIVE Downstream(raw_state, prev, elapsed, step_time, new_elapsed, new_time_since_visit)
            AS (
            -- Anchor definition = the updated states
            SELECT raw_state, prev, elapsed, step_time, elapsed AS new_elapsed, time_since_visit as new_time_since_visit
                FROM db_states
                WHERE raw_state in ({})
            UNION
            -- Recursive definition = the states that point to earlier states via prev
            SELECT db.raw_state, db.prev, db.elapsed, db.step_time, prior.new_elapsed + db.step_time AS new_elapsed,
                IF(db.time_since_visit = 0, 0, prior.new_time_since_visit + db.step_time) AS new_time_since_visit
            FROM db_states AS db
            INNER JOIN Downstream AS prior
                ON db.prev = prior.raw_state
            WHERE prior.new_elapsed + db.step_time < db.elapsed
            )
            "#,
            vec!["?"; n].join(", ")
        ))
    }

    pub fn test_downstream(n: usize) -> SqlQuery {
        downstream(n).sql(
            r#"
            SELECT raw_state, elapsed AS old_elapsed, new_elapsed, new_time_since_visit, step_time FROM Downstream
            ORDER BY new_elapsed
            "#,
        )
    }

    pub fn improve_downstream(n: usize) -> SqlQuery {
        downstream(n).sql(
            r#"
            -- Perform the update on the states we've selected and paired with improved times
            UPDATE db_states
            INNER JOIN Downstream AS res ON db_states.raw_state = res.raw_state
            SET db_states.elapsed = res.new_elapsed, db_states.time_since_visit = res.new_time_since_visit
            "#,
        )
    }
}
