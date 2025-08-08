use crate::context::{ContextWrapper, Ctx, Wrapper};
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

#[derive(Default, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::db_states, check_for_backend(Mysql))]
pub struct DBState {
    pub raw_state: Vec<u8>,
    pub progress: u32,
    pub elapsed: u32,
    pub time_since_visit: u32,
    pub estimated_remaining: u32,
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
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Could not create table");
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
                elapsed.eq(sqlif(elapsed.gt(new_elapsed), new_elapsed, elapsed)),
                time_since_visit.eq(sqlif(
                    elapsed.gt(new_elapsed),
                    insertvalues(time_since_visit),
                    time_since_visit,
                )),
                hist.eq(sqlif(elapsed.gt(new_elapsed), insertvalues(hist), hist)),
                prev.eq(sqlif(elapsed.gt(new_elapsed), insertvalues(prev), prev)),
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
                elapsed.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(elapsed),
                    elapsed,
                )),
                time_since_visit.eq(sqlif(
                    elapsed.gt(insertvalues(elapsed)),
                    insertvalues(time_since_visit),
                    time_since_visit,
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

    pub fn get_best_times(&mut self, state: &T) -> QueryResult<BestTimes> {
        queries::get_best_times(&serialize_state(state)).first(&mut self.conn)
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
}

#[allow(unused)]
mod queries {
    use crate::models::DBState;
    use crate::schema::db_states::dsl::*;
    use crate::scoring::BestTimes;
    use diesel::dsl::{auto_type, exists, select, AsSelect};
    use diesel::mysql::Mysql;
    use diesel::prelude::*;
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

    /*
    Recursive query for history
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

    If the first result isn't the starting state, there's a recursive loop in the states
    that needs to be fixed.

    Recursive update
    WITH RECURSIVE Downstream(raw_state, prev, elapsed, step_time, new_elapsed)
    AS (
      -- Anchor definition = the updated state
      SELECT raw_state, prev, elapsed, step_time, elapsed AS new_elapsed
        FROM db_states
        WHERE raw_state = ?
      UNION DISTINCT
      -- Recursive definition = the states that point to earlier states via prev
      SELECT db.raw_state, db.prev, db.elapsed, db.step_time, prior.elapsed + db.step_time AS new_elapsed
      FROM db_states AS db
      INNER JOIN Downstream AS prior
        ON db.prev = prior.raw_state
    )
    UPDATE Downstream
    SET elapsed = new_elapsed
    */
}
