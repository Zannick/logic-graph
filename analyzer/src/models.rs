use crate::context::{ContextWrapper, Ctx, HistoryAlias, Wrapper};
use crate::db::{get_obj_from_data, serialize_data, serialize_state, NextSteps};
use crate::schema::db_states::dsl::*;
use crate::scoring::{BestTimes, EstimatorWrapper, ScoreMetric};
use crate::world::{Location, World};
use diesel::dsl::{not, DuplicatedKeys};
use diesel::expression::functions::*;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::sql_types::*;
use dotenvy::dotenv;
use rayon::prelude::*;
use std::env;
use std::marker::PhantomData;

define_sql_function!(
    #[sql_name = "IF"]
    fn sqlif<T: SingleValue>(cond: Bool, case_true: T, case_false: T) -> T
);
define_sql_function! (
    #[sql_name = "VALUES"]
    fn insertvalues<T: SingleValue>(v: T) -> T
);

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
        let h = ctx.recent_history();
        DBState {
            raw_state: serialize_state(ctx.get()),
            progress: ctx.get().progress(),
            elapsed: ctx.elapsed(),
            time_since_visit: ctx.time_since_visit(),
            estimated_remaining: metric.estimated_remaining_time(ctx.get()),
            step_time: ctx.recent_dur(),
            won: is_solution,
            hist: if h.is_empty() {
                None
            } else {
                Some(serialize_data(h))
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
    use crate::db::get_obj_from_data;
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
        pub hist: Option<Vec<HistoryAlias<T>>>,
        pub prev: Option<T>,
        pub next_steps: Option<Vec<Vec<HistoryAlias<T>>>>,
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
        pub hist: Option<Vec<HistoryAlias<T>>>,
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

pub struct MySQLDB<'w, W, T, const KS: usize, SM> {
    conn: MysqlConnection, // TODO: connection pool?
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
        Self {
            conn: establish_connection(),
            metric,
            phantom: PhantomData::default(),
        }
    }

    /// Opens a DB connection and starts a test transaction to it, ensuring that no changes are made during the test.
    pub fn with_test_connection(metric: SM) -> Self {
        use diesel_migrations::*;
        const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

        let mut conn =
            MysqlConnection::establish("mysql://logic_graph@localhost/logic_graph__unittest")
                .expect("Could not connect to mysql server");
        // dropping the table will fail if it doesn't exist
        let _ = conn.revert_all_migrations(MIGRATIONS);
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        conn.begin_test_transaction().unwrap();
        Self {
            conn,
            metric,
            phantom: PhantomData::default(),
        }
    }

    // test-only, for making raw queries
    pub fn connection(&mut self) -> &mut MysqlConnection {
        &mut self.conn
    }

    pub fn metric(&self) -> &SM {
        &self.metric
    }

    pub fn insert_one(
        &mut self,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
    ) -> QueryResult<usize> {
        let value = DBState::from_ctx(ctx, is_solution, serialized_prev, queue, &self.metric);
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
            .execute(&mut self.conn)
    }

    pub fn insert_batch(&mut self, values: Vec<DBState>) -> QueryResult<usize> {
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
            .execute(&mut self.conn)
    }

    /// Insert/update both a processed state and its subsequent states.
    /// Assumes the processed state has already been committed.
    /// It's assumed that all subsequent states will be queued if new.
    pub fn insert_processed(
        &mut self,
        ctx: &ContextWrapper<T>,
        world: &W,
        next_states: &Vec<ContextWrapper<T>>,
    ) -> QueryResult<usize> {
        let parent_state = serialize_state(ctx.get());
        let mut next_hists = Vec::new();
        for next_state in next_states {
            next_hists.push(next_state.recent_history());
        }
        let values = next_states
            .into_par_iter()
            .map(|s| {
                DBState::from_ctx(
                    s,
                    world.won(s.get()),
                    Some(parent_state.clone()),
                    true,
                    &self.metric,
                )
            })
            .collect::<Vec<_>>();

        let inserts = self.insert_batch(values)?;

        // Update in a separate command so we don't need to add conditions on these sets for the above
        // insert-on-duplicate-keys-update entries which will never need to update these fields.
        let updates = diesel::update(db_states.filter(raw_state.eq(parent_state)))
            .set((
                processed.eq(true),
                next_steps.eq(serialize_data(next_hists)),
            ))
            .execute(&mut self.conn)?;
        Ok(inserts + updates)
    }

    pub fn get_record(&mut self, state: &T) -> QueryResult<DBEntry<T>> {
        queries::lookup_state(&serialize_state(state))
            .first(&mut self.conn)
            .map(|row: DBState| row.into())
    }

    pub fn get_best_times(&mut self, state: &T) -> QueryResult<BestTimes> {
        queries::get_best_times(&serialize_state(state)).first(&mut self.conn)
    }

    pub fn get_prev_hist(
        &mut self,
        state: &T,
    ) -> QueryResult<(Option<T>, Option<Vec<HistoryAlias<T>>>)> {
        queries::get_prev_hist(&serialize_state(state))
            .first(&mut self.conn)
            .map(|(p, h): (Option<Vec<u8>>, Option<Vec<u8>>)| {
                (
                    p.map(|buf| get_obj_from_data(&buf).unwrap()),
                    h.map(|buf| get_obj_from_data(&buf).unwrap()),
                )
            })
    }

    pub fn get_next_steps(&mut self, state: &T) -> QueryResult<Option<NextSteps<T>>> {
        queries::get_next_steps(&serialize_state(state))
            .first(&mut self.conn)
            .map(|data: Option<Vec<u8>>| {
                if let Some(buf) = &data {
                    Some(get_obj_from_data::<NextSteps<T>>(buf).unwrap())
                } else {
                    None
                }
            })
    }

    pub fn has_next_steps(&mut self, state: &T) -> QueryResult<bool> {
        queries::has_next_steps(&serialize_state(state)).first(&mut self.conn)
    }

    pub fn exists(&mut self, state: &T) -> QueryResult<bool> {
        queries::row_exists(&serialize_state(state)).first(&mut self.conn)
    }

    pub fn full_history(&mut self, state: &T) -> QueryResult<Vec<HistoryEntry<T>>> {
        queries::full_history()
            .bind::<Blob, _>(&serialize_state(state))
            .load(&mut self.conn)
            .map(|vec| vec.into_iter().map(|hs: HistoryState| hs.into()).collect())
    }

    /// Recursively update the times for the states downstream from the given states.
    /// 
    /// This is guaranteed safe and correct if no given state that had an improvement to pass
    /// downstream is in another given state's downstream, which should automatically be the case as long as
    /// as the given states passed in together are from the same "next" cohort of a processed state.
    pub fn improve_downstream(&mut self, states: &[&T]) -> QueryResult<usize> {
        let mut q = queries::improve_downstream(states.len()).into_boxed();
        for state in states {
            q = q.bind::<Blob, _>(serialize_state(*state))
        }
        q.execute(&mut self.conn)
    }

    pub fn test_downstream(&mut self, states: &[&T]) -> QueryResult<Vec<DownstreamEntry<T>>> {
        let mut q = queries::test_downstream(states.len()).into_boxed();
        for state in states {
            q = q.bind::<Blob, _>(serialize_state(*state));
        }
        q.load(&mut self.conn).map(|vec| {
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
    pub fn lookup_state<'a>(key: &'a Vec<u8>) -> _ {
        db_states.find(key)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_best_times<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select::<AsSelect<BestTimes, Mysql>>(BestTimes::as_select())
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_next_steps<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(next_steps)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn has_next_steps<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(next_steps.is_not_null())
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_processed<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(processed)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_prev_hist<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select((prev, hist))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_elapsed_prev_hist<'a>(key: &'a Vec<u8>) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select((elapsed, prev, hist))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn row_exists<'a>(key: &'a Vec<u8>) -> _ {
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
                IF(db.time_since_visit = 0, 0, prior.time_since_visit + db.step_time) AS new_time_since_visit
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
