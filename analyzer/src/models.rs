use crate::context::{ContextWrapper, Ctx, Wrapper};
use crate::db::{get_obj_from_data, serialize_data, serialize_state, NextSteps};
use crate::schema::db_states::dsl::*;
use crate::scoring::{BestTimes, ScoreMetric};
use crate::world::{Location, World};
use diesel::dsl::{not, DuplicatedKeys};
use diesel::expression::functions::*;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::sql_types::*;
use dotenvy::dotenv;
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

    pub fn encode(
        &self,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
    ) -> DBState {
        let h = ctx.recent_history();
        DBState {
            raw_state: serialize_state(ctx.get()),
            progress: ctx.get().progress(),
            elapsed: ctx.elapsed(),
            time_since_visit: ctx.time_since_visit(),
            estimated_remaining: self.metric.estimated_remaining_time(ctx.get()),
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

    pub fn insert_one(
        &mut self,
        ctx: &ContextWrapper<T>,
        is_solution: bool,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
    ) -> QueryResult<usize> {
        let value = self.encode(ctx, is_solution, serialized_prev, queue);
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

    /// Insert/update both a processed state and its subsequent states.
    /// It's assumed that all subsequent states will be queued if new.
    pub fn insert_processed(
        &mut self,
        ctx: &ContextWrapper<T>,
        world: &W,
        next_states: &Vec<ContextWrapper<T>>,
    ) -> QueryResult<usize> {
        let parent_state = serialize_state(ctx.get());
        let mut next_hists = Vec::new();
        let mut values = Vec::new();
        for next_state in next_states {
            next_hists.push(next_state.recent_history());
            values.push(self.encode(
                next_state,
                world.won(next_state.get()),
                Some(parent_state.clone()),
                true,
            ));
        }

        let inserts = diesel::insert_into(db_states)
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
            .execute(&mut self.conn)?;

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

mod queries {
    use crate::models::DBState;
    use crate::schema::db_states::dsl::*;
    use crate::scoring::BestTimes;
    use diesel::dsl::{AsSelect, Find, IsNotNull, Select};
    use diesel::mysql::Mysql;
    use diesel::prelude::*;
    use diesel::sql_types::*;

    type Lookup<'a> = Find<db_states, &'a Vec<u8>>;
    type GetBest<'a> = Select<Lookup<'a>, AsSelect<BestTimes, Mysql>>;
    type GetNextSteps<'a> = Select<Lookup<'a>, next_steps>;
    type HasNextSteps<'a> = Select<Lookup<'a>, IsNotNull<next_steps>>;
    type GetProcessed<'a> = Select<Lookup<'a>, processed>;
    type GetPrevHist<'a> = Select<Lookup<'a>, (prev, hist)>;
    type GetElapsedPrevHist<'a> = Select<Lookup<'a>, (elapsed, prev, hist)>;

    define_sql_function!(
        #[sql_name = "IF"]
        fn sqlif<T: SingleValue>(cond: Bool, case_true: T, case_false: T) -> T
    );
    define_sql_function! (
        #[sql_name = "VALUES"]
        fn insertvalues<T: SingleValue>(v: T) -> T
    );

    pub fn lookup_state(key: &Vec<u8>) -> Lookup {
        db_states.find(key)
    }

    pub fn get_best_times(key: &Vec<u8>) -> GetBest {
        lookup_state(key).select(BestTimes::as_select())
    }

    pub fn get_next_steps(key: &Vec<u8>) -> GetNextSteps {
        db_states.find(key).select(next_steps)
    }

    pub fn has_next_steps(key: &Vec<u8>) -> HasNextSteps {
        db_states.find(key).select(next_steps.is_not_null())
    }

    pub fn get_processed(key: &Vec<u8>) -> GetProcessed {
        lookup_state(key).select(processed)
    }

    pub fn get_prev_hist(key: &Vec<u8>) -> GetPrevHist {
        lookup_state(key).select((prev, hist))
    }

    pub fn get_elapsed_prev_hist(key: &Vec<u8>) -> GetElapsedPrevHist {
        lookup_state(key).select((elapsed, prev, hist))
    }

    #[diesel::dsl::auto_type]
    pub fn is_queued() -> _ {
        queued.eq(true)
    }

    #[diesel::dsl::auto_type]
    pub fn unprocessed() -> _ {
        processed.eq(false)
    }

    /*
    Recursive query for history
    WITH FullHistory(raw_state, prev, hist, elapsed, step)
    AS (
       -- Anchor definition = the end state
       SELECT raw_state, prev, hist, elapsed, 0 AS step
         FROM db_states
         WHERE raw_state = ?
       UNION ALL
       -- Recursive definition = the state pointed to by the previous state's prev
       SELECT raw_state, prev, hist, elapsed, step + 1
       FROM db_states as db
       INNER JOIN FullHistory as fh
           ON db.raw_state = fh.prev
    )
    SELECT raw_state, prev, hist, elapsed, step
    FROM FullHistory
    ORDER BY step DESC  -- first state to last
    */
}
