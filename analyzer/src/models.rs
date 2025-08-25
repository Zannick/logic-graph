use crate::context::{ContextWrapper, Ctx, HistoryAlias, Wrapper};
use crate::db::HeapMetric;
use crate::estimates::UNREASONABLE_TIME;
use crate::schema::db_states::dsl::*;
use crate::scoring::{BestTimes, EstimatorWrapper, ScoreMetric};
use crate::storage::{
    get_obj_from_data, serialize_data, serialize_state, CachedEstimates, ContextDB,
};
use crate::world::{Exit, Location, Warp, World};
use anyhow::Result;
use diesel::dsl::{max, min, not, DuplicatedKeys};
use diesel::expression::functions::*;
use diesel::mysql::Mysql;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error::NotFound;
use diesel::sql_types::*;
use dotenvy::dotenv;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::env;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use textplots::{Chart, Plot, Shape};

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
define_sql_function!(
    #[sql_name = "FLOOR"]
    fn floor(val: Float) -> Integer;
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

#[derive(Debug, Default, Queryable, Selectable, Insertable, Eq, PartialEq, Ord, PartialOrd)]
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
}

impl DBState {
    pub fn from_ctx<'w, W, T>(
        ctx: &ContextWrapper<T>,
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
            serialized_prev,
            queue,
            metric.estimated_remaining_time(ctx.get()),
        )
    }

    pub fn from_ctx_with_raw<W, T>(
        raw: Vec<u8>,
        ctx: &ContextWrapper<T>,
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
            won: ctx.won(),
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
            queued: !ctx.won() && queue,
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

    #[derive(QueryableByName)]
    #[diesel(check_for_backend(Mysql))]
    pub struct Bucket {
        #[diesel(sql_type = Integer)]
        pub bucket_id: i32,
        #[diesel(sql_type = BigInt)]
        pub count: i64,
    }
}
pub use q::*;

pub type MysqlPoolConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub type StickyConnection = Option<MysqlPoolConnection>;

pub struct MySQLDB<'w, W, T, const KS: usize, SM> {
    pool: Pool<ConnectionManager<MysqlConnection>>,
    metric: SM,
    phantom: PhantomData<&'w (W, T)>,
    max_time: AtomicU32,
    recovery: AtomicBool,
    cached_estimates: CachedEstimates,
}

impl<'w, W, T, L, E, const KS: usize, SM> HeapMetric for MySQLDB<'w, W, T, KS, SM>
where
    W: World<Location = L, Exit = E> + 'w,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
    SM: ScoreMetric<'w, W, T, KS>,
{
    type Score = SM::Score;
}

impl<'w, W, T, const KS: usize, SM> ContextDB<'w, W, T, KS, SM> for MySQLDB<'w, W, T, KS, SM>
where
    W: World + 'w,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    SM: ScoreMetric<'w, W, T, KS> + 'w,
{
    const NAME: &'static str = "MySQLDB";

    fn metric(&self) -> &SM {
        &self.metric
    }

    // region: Stats
    fn len(&self) -> usize {
        self.cached_estimates.size.load(Ordering::Acquire)
    }

    fn seen(&self) -> usize {
        self.cached_estimates.seen.load(Ordering::Acquire)
    }

    fn processed(&self) -> usize {
        self.cached_estimates.processed.load(Ordering::Acquire)
    }

    fn preserved_best(&self, prog: usize) -> u32 {
        self.cached_estimates.min_estimates[prog].load(Ordering::Acquire)
    }

    fn preserved_bests(&self) -> Vec<u32> {
        self.cached_estimates
            .min_estimates
            .iter()
            .map(|a| a.load(Ordering::Acquire))
            .collect()
    }

    fn min_preserved_progress(&self) -> Option<usize> {
        self.cached_estimates
            .min_estimates
            .iter()
            .position(|a| a.load(Ordering::Acquire) != u32::MAX)
    }

    fn print_graphs(&self) -> Result<()> {
        // graphs the other db provides:
        // preserved elements count by elapsed time (histogram)
        // preserved elements estimated time by elapsed time (histogram)

        // other potential ideas
        // processed % per progress level
        // time_since heatmap per progress
        let mut conn = self.get_sticky_connection();
        let (max_e, min_e) = db_states
            .filter(processed.eq(false))
            .select((max(elapsed), min(elapsed)))
            .first::<(Option<u32>, Option<u32>)>(self.sticky(&mut conn))
            .unwrap();
        let Some(max_e) = max_e else {
            println!("No unprocessed states in sql db");
            return Ok(());
        };
        let max_e = max_e as f32;
        let min_e = min_e.unwrap() as f32;
        let num_bins = 70f32;
        let bucket_width = (max_e - min_e + 1.0) / num_bins;
        let time_buckets = diesel::sql_query(
            r#"
            SELECT
                FLOOR(elapsed / ?) AS bucket,
                COUNT(*)
            FROM db_states
            WHERE
                processed = FALSE
            GROUP BY bucket
            ORDER BY bucket
        "#,
        )
        .bind::<Float, _>(bucket_width)
        .load::<Bucket>(self.sticky(&mut conn))?;

        // We may be missing empty buckets
        let mut bins = vec![0.0f32; time_buckets.last().unwrap().bucket_id as usize + 1];
        for bucket in time_buckets {
            bins[bucket.bucket_id as usize] = bucket.count as f32;
        }
        Chart::new(132, 28, 0.0, max_e)
            .lineplot(&Shape::Bars(
                &bins
                    .into_iter()
                    .enumerate()
                    .map(|(bn, ct)| (min_e + (bn as f32) * bucket_width, ct))
                    .collect::<Vec<_>>(),
            ))
            .nice();

        Ok(())
    }

    fn extra_stats(&self) -> String {
        // the rocksdb counts skips by the db manager itself, as well as
        // readds (improvements to a state that brings it under max time)
        // and background deletes from the queue.
        // we don't need to count any of that here.
        let mut conn = self.get_sticky_connection();

        // this is similar, though.
        let over_limit = db_states
            .filter(
                (elapsed + estimated_remaining)
                    .ge(self.max_time())
                    .and(not(processed)),
            )
            .count()
            .get_result::<i64>(self.sticky(&mut conn))
            .unwrap();
        let broken = db_states
            .filter((elapsed + estimated_remaining).ge(UNREASONABLE_TIME))
            .count()
            .get_result::<i64>(self.sticky(&mut conn))
            .unwrap();
        format!("over time: {}; unreasonable: {}", over_limit, broken)
    }

    fn reset_all_cached_estimates(&self) {
        self.cached_estimates.size.store(
            db_states
                .filter(queries::preserved())
                .count()
                .get_result::<i64>(&mut self.pool_connection())
                .unwrap() as usize,
            Ordering::SeqCst,
        );
        self.cached_estimates.seen.store(
            db_states
                .count()
                .get_result::<i64>(&mut self.pool_connection())
                .unwrap() as usize,
            Ordering::SeqCst,
        );
        self.cached_estimates.processed.store(
            db_states
                .filter(processed.eq(true))
                .count()
                .get_result::<i64>(&mut self.pool_connection())
                .unwrap() as usize,
            Ordering::SeqCst,
        );

        let mut bests = vec![u32::MAX; W::NUM_CANON_LOCATIONS + 1];
        for (prog, score) in db_states
            .filter(queries::available(self.max_time()))
            .group_by(progress)
            // Ideally this would be changeable to the current MetricType primary
            // (we can't do min on a tuple) but writing a type is hard. TODO: features?
            .select((progress, min(time_since_visit)))
            .get_results::<(u32, Option<u32>)>(&mut self.pool_connection())
            .unwrap()
        {
            if let Some(sc) = score {
                bests[prog as usize] = sc;
            }
        }
        for (est, min) in self.cached_estimates.min_estimates.iter().zip(bests) {
            est.store(min, Ordering::SeqCst);
        }
    }
    // endregion: Stats

    // region: Time

    fn max_time(&self) -> u32 {
        self.max_time.load(Ordering::Acquire)
    }
    fn set_max_time(&self, max_time: u32) {
        self.max_time.fetch_min(max_time, Ordering::Release);
    }
    // endregion

    // region: Reads

    fn get_best_times_raw(&self, state_key: &[u8]) -> Result<BestTimes> {
        Ok(queries::get_best_times(state_key).first(&mut self.pool_connection())?)
    }

    fn estimated_remaining_time(&self, ctx: &T) -> u32 {
        queries::get_estimate(&serialize_state(ctx))
            .first(&mut self.pool_connection())
            .unwrap_or_else(|err| {
                assert!(
                    matches!(err, NotFound),
                    "Unexpected error reading estimated_remaining: {}",
                    err
                );
                // We don't initiate a write to db in this case
                // because we assume the state will be added directly later
                self.metric.estimated_remaining_time(ctx)
            })
    }

    fn was_processed_raw(&self, key: &[u8]) -> Result<bool> {
        match queries::get_processed(key).first(&mut self.pool_connection()) {
            Err(NotFound) => Ok(false),
            Ok(b) => Ok(b),
            Err(s) => Err(s)?,
        }
    }

    fn get_best_times_processed_raw(&self, state_key: &[u8]) -> Result<(BestTimes, bool)> {
        Ok(queries::get_best_times_processed(state_key).first(&mut self.pool_connection())?)
    }

    fn get_history_raw(&self, state_key: &Vec<u8>) -> Result<(Vec<HistoryAlias<T>>, u32)> {
        let entries = self.full_history_raw(state_key, &mut self.get_sticky_connection())?;
        let total_time = entries.last().map_or(0, |entry| entry.elapsed);
        Ok((
            entries.into_iter().filter_map(|entry| entry.hist).collect(),
            total_time,
        ))
    }

    fn get_last_history_step(&self, el: &T) -> Result<Option<HistoryAlias<T>>> {
        let hist_raw = queries::lookup_state(&serialize_state(el))
            .select(hist)
            .first::<Option<Vec<u8>>>(&mut self.pool_connection())?;
        Ok(hist_raw.map(|r| get_obj_from_data(&r).unwrap()))
    }
    // endregion

    // region: Writes

    fn push(&self, el: ContextWrapper<T>, parent: Option<&T>) -> Result<()> {
        let mut conn = self.get_sticky_connection();
        self.insert_one(&el, parent.map(|p| serialize_state(p)), false, &mut conn)?
            .1;
        Ok(())
    }

    fn pop(&self, start_progress: usize) -> Result<Option<ContextWrapper<T>>> {
        let mut conn = self.get_sticky_connection();
        let (state, best) = match queries::best_available(1, start_progress as u32, self.max_time())
            .select((raw_state, BestTimes::as_select()))
            .first::<(Vec<u8>, BestTimes)>(self.sticky(&mut conn))
        {
            Ok(s) => s,
            Err(NotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let ctx = ContextWrapper::<T>::with_times(
            get_obj_from_data(&state).unwrap(),
            best.elapsed,
            best.time_since_visit,
        );
        self.cached_estimates.reset_estimates_in_range(
            start_progress,
            ctx.get().count_visits(),
            SM::score_primary(SM::score_from_times(best)),
        );
        diesel::update(queries::lookup_state(&state))
            .set(queued.eq(true))
            .execute(self.sticky(&mut conn))?;
        self.cached_estimates.size.fetch_sub(1, Ordering::Release);
        Ok(Some(ctx))
    }

    fn evict(&self, iter: impl IntoIterator<Item = (T, SM::Score)>) -> Result<()> {
        // We don't have the full state information, so instead we'll assume that all evicted items
        // have been stored previously. As such, just set them all as unqueued.
        let mut mins = vec![u32::MAX; W::NUM_CANON_LOCATIONS + 1];
        let res = diesel::update(queries::lookup_many(
            &iter
                .into_iter()
                .map(|(t, sc)| {
                    let p = t.count_visits();
                    mins[p] = std::cmp::min(mins[p], SM::score_primary(sc));
                    serialize_state(&t)
                })
                .collect::<Vec<_>>(),
        ))
        .set(queued.eq(false))
        .execute(&mut self.pool_connection())?;

        for (est, min) in self
            .cached_estimates
            .min_estimates
            .iter()
            .zip(mins.into_iter())
        {
            est.fetch_min(min, Ordering::Release);
        }
        self.cached_estimates.size.fetch_add(res, Ordering::Release);
        Ok(())
    }

    fn retrieve(
        &self,
        start_progress: usize,
        count: usize,
        score_limit: u32,
    ) -> Result<Vec<(T, SM::Score)>> {
        let start = Instant::now();
        let mut conn = self.get_sticky_connection();
        let sts = queries::best_available(count as i64, start_progress as u32, self.max_time())
            // score limit applies to the score primary which for now is time_since for TimeSinceAndElapsed
            .filter(time_since_visit.le(score_limit))
            .select((raw_state, BestTimes::as_select()))
            .load::<(Vec<u8>, BestTimes)>(self.sticky(&mut conn))?;
        let res: Vec<(T, SM::Score)> = sts
            .iter()
            .map(|(rs, bests)| {
                (
                    get_obj_from_data(&rs).unwrap(),
                    SM::score_from_times(*bests),
                )
            })
            .collect();

        if let Some((t, sc)) = res.last() {
            self.cached_estimates.reset_estimates_in_range(
                start_progress,
                t.count_visits(),
                SM::score_primary(*sc),
            );
        } else {
            // No results
            self.cached_estimates
                .reset_estimates_in_range_unbounded(start_progress);
            return Ok(res);
        }

        // After reconstructing the state objects, we can reuse the raw states to update the db.
        let just_states = sts.into_iter().map(|(s, _)| s).collect::<Vec<_>>();
        diesel::update(queries::lookup_many(&just_states))
            .set(queued.eq(true))
            .execute(self.sticky(&mut conn))?;
        log::debug!(
            "Retrieved {} elements from mysql in {:?}",
            res.len(),
            start.elapsed()
        );
        self.cached_estimates
            .size
            .fetch_sub(just_states.len(), Ordering::Release);
        Ok(res)
    }

    fn record_one(
        &self,
        el: &mut ContextWrapper<T>,
        parent: Option<&T>,
    ) -> Result<Option<SM::Score>> {
        let mut conn = self.get_sticky_connection();
        let dbst = self
            .insert_one(&el, parent.map(|p| serialize_state(p)), true, &mut conn)?
            .0;
        let best_times = queries::get_best_times(&dbst.raw_state).first(self.sticky(&mut conn))?;
        // estimated time cannot change, so we only have to compare elapsed

        Ok(Some(SM::score_from_times(best_times)))
    }

    fn record_processed(
        &self,
        parent: &T,
        states: &mut Vec<ContextWrapper<T>>,
    ) -> Result<Vec<Option<SM::Score>>> {
        let mut conn = self.get_sticky_connection();
        let dbsts = self.insert_processed(parent, states, &mut conn)?.0;
        // TODO: pass serialized states by reference? like &[&Vec<u8>] instead of &[Vec<u8>]
        let keys = dbsts
            .iter()
            .map(|dbst| dbst.raw_state.clone())
            .collect::<Vec<_>>();

        // Retrieve the best times of each child state to see if we have the best time
        let emap = FxHashMap::from_iter(
            queries::get_bests(&keys).load::<(Vec<u8>, BestTimes)>(self.sticky(&mut conn))?,
        );
        Ok(dbsts
            .into_iter()
            .map(|dbst| {
                let new_best = emap.get(&dbst.raw_state).unwrap();
                if dbst.elapsed > new_best.elapsed {
                    // not improved
                    None
                } else {
                    Some(SM::score_from_times(*new_best))
                }
            })
            .collect())
    }

    fn cleanup(&self, _exit_signal: &AtomicBool) -> Result<()> {
        self.reset_all_cached_estimates();
        Ok(())
    }

    fn recovery(&self) -> bool {
        self.recovery.load(Ordering::Acquire)
    }

    fn restore(&self) {
        log::debug!("Starting restore");
        self.recovery.store(true, Ordering::Release);
        diesel::update(db_states.filter(queued.eq(true)))
            .set(queued.eq(false))
            .execute(&mut self.pool_connection())
            .unwrap();
        self.reset_all_cached_estimates();
        self.recovery.store(false, Ordering::Release);
        log::debug!(
            "Finished restore: mysql db ready for retrievals with {} elements",
            self.len()
        );
    }
    // endregion
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
                .max_size(rayon::current_num_threads() as u32 * 2)
                .build(manager)
                .expect("Could not build MySQL connection pool"),
            metric,
            phantom: PhantomData::default(),
            max_time: u32::MAX.into(),
            recovery: false.into(),
            cached_estimates: CachedEstimates::new(W::NUM_CANON_LOCATIONS + 1),
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
            max_time: u32::MAX.into(),
            recovery: false.into(),
            cached_estimates: CachedEstimates::new(W::NUM_CANON_LOCATIONS + 1),
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
        conn.get_or_insert_with(|| self.pool_connection())
    }

    fn pool_connection(&self) -> MysqlPoolConnection {
        let start = Instant::now();
        let p = self.pool.get().expect("Failed to get a pool connection");
        if start.elapsed() > Duration::from_secs(1) {
            log::debug!("Long delay for a connection: {:?}", start.elapsed());
        }
        p
    }

    pub fn metric(&self) -> &SM {
        &self.metric
    }

    pub fn encode_one_for_upsert(
        &self,
        ctx: &ContextWrapper<T>,
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
        DBState::from_ctx_with_raw(key, ctx, serialized_prev, queue, est)
    }

    pub fn encode_many_for_upsert(
        &self,
        ctxs_prevs: Vec<(&ContextWrapper<T>, Option<Vec<u8>>)>,
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
                DBState::from_ctx_with_raw(key, ctx, sprev, queue, est)
            })
            .collect()
    }

    // Constructs a row for the given state, commits it to the database, and returns it to the caller.
    pub fn insert_one(
        &self,
        ctx: &ContextWrapper<T>,
        serialized_prev: Option<Vec<u8>>,
        queue: bool,
        conn: &mut StickyConnection,
    ) -> QueryResult<(DBState, usize)> {
        let value = self.encode_one_for_upsert(ctx, serialized_prev, queue, conn);
        let new_elapsed = value.elapsed;
        let res = diesel::insert_into(db_states)
            .values(&value)
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
            .execute(self.sticky(conn))?;

        // a value of 2 means the row was updated
        if res == 1 {
            if !queue {
                self.cached_estimates.size.fetch_add(1, Ordering::Release);
            }
            self.cached_estimates.seen.fetch_add(1, Ordering::Release);
        }
        Ok((value, res))
    }

    pub fn insert_batch(
        &self,
        values: &Vec<DBState>,
        conn: &mut StickyConnection,
    ) -> QueryResult<usize> {
        let start = Instant::now();
        let res = diesel::insert_into(db_states)
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
                // progress, estimated_remaining, won
            ))
            .execute(self.sticky(conn))?;
        // result = 1 for insert, 2 for update, so res - values.len() = number of updated keys.
        let inserts = (values.len() * 2).saturating_sub(res);
        if inserts > 0 {
            self.cached_estimates
                .seen
                .fetch_add(inserts, Ordering::Release);
            // we shouldn't have a mix of queuing
            if !values[0].queued {
                self.cached_estimates
                    .size
                    .fetch_add(inserts, Ordering::Release);
            }
        }
        if start.elapsed() > Duration::from_secs(1) {
            log::debug!("Long batch insert write: {:?}", start.elapsed());
        }
        Ok(res)
    }

    /// Insert/update both a processed state and its subsequent states.
    /// Assumes the processed state has already been committed.
    /// It's assumed that all subsequent states will be queued if new.
    pub fn insert_processed(
        &self,
        proc: &T,
        next_states: &Vec<ContextWrapper<T>>,
        conn: &mut StickyConnection,
    ) -> QueryResult<(Vec<DBState>, usize)> {
        let parent_state = serialize_state(proc);
        let mut values = self.encode_many_for_upsert(
            next_states
                .iter()
                .zip(std::iter::repeat(Some(&parent_state).cloned()))
                .collect(),
            true,
            conn,
        );
        values.sort();

        let inserts = self.insert_batch(&values, conn)?;

        // Update in a separate command so we don't need to add conditions on these sets for the above
        // insert-on-duplicate-keys-update entries which will never need to update these fields.
        let updates = diesel::update(queries::lookup_state(&parent_state))
            .set((processed.eq(true),))
            .execute(self.sticky(conn))?;
        self.cached_estimates
            .processed
            .fetch_add(1, Ordering::Release);
        Ok((values, inserts + updates))
    }

    pub fn insert_processed_and_improve(
        &self,
        proc: &T,
        next_states: &Vec<ContextWrapper<T>>,
    ) -> QueryResult<(Vec<DBState>, usize)> {
        let mut conn = self.get_sticky_connection();
        let (values, count) = self.insert_processed(proc, next_states, &mut conn)?;

        // Should we improve from proc only, rather than its child states, in case proc was improved
        // while we were processing it? The rocksdb code looks up the processed state to check for improvements
        // before recording the child states.
        let count = count + self.improve_downstream(proc, &mut conn)?;
        Ok((values, count))
    }

    pub fn get_record(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<DBEntry<T>> {
        queries::lookup_state(&serialize_state(state))
            .first(self.sticky(conn))
            .map(|row: DBState| row.into())
    }

    pub fn get_estimates(
        &self,
        keys: &[Vec<u8>],
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<(Vec<u8>, u32)>> {
        queries::get_estimates(keys).load(self.sticky(conn))
    }

    pub fn get_best_conn(&self, state: &T, conn: &mut StickyConnection) -> Result<BestTimes> {
        self.get_best_conn_raw(&serialize_state(state), conn)
    }

    pub fn get_best_conn_raw(
        &self,
        state_key: &[u8],
        conn: &mut StickyConnection,
    ) -> Result<BestTimes> {
        Ok(queries::get_best_times(state_key).first(self.sticky(conn))?)
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

    /// Recursively update the times for the states downstream from the given state.
    pub fn improve_downstream(&self, state: &T, conn: &mut StickyConnection) -> QueryResult<usize> {
        self.improve_downstream_raw(&serialize_state(state), conn)
    }

    /// Recursively update the times for the states downstream from the given encoded states.
    pub fn improve_downstream_raw(
        &self,
        state: &[u8],
        conn: &mut StickyConnection,
    ) -> QueryResult<usize> {
        queries::improve_downstream()
            .bind::<Blob, _>(&state)
            .execute(self.sticky(conn))
    }

    pub fn test_downstream(
        &self,
        state: &T,
        conn: &mut StickyConnection,
    ) -> QueryResult<Vec<DownstreamEntry<T>>> {
        queries::test_downstream()
            .bind::<Blob, _>(serialize_state(state))
            .load(self.sticky(conn))
            .map(|vec| {
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
    pub fn lookup_many<'a>(keys: &'a [Vec<u8>]) -> _ {
        db_states.filter(raw_state.eq_any(keys))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_best_times<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select::<AsSelect<BestTimes, Mysql>>(BestTimes::as_select())
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_processed<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select(processed)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_best_times_processed<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        row.select::<(AsSelect<BestTimes, Mysql>, processed)>((BestTimes::as_select(), processed))
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
    pub fn row_exists<'a>(key: &'a [u8]) -> _ {
        let row: LookupState<'a> = lookup_state(key);
        select(exists(row.select(1i32.into_sql::<Integer>())))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_estimates<'a>(keys: &'a [Vec<u8>]) -> _ {
        let rows: LookupMany<'a> = lookup_many(keys);
        rows.select((raw_state, estimated_remaining))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn get_bests<'a>(keys: &'a [Vec<u8>]) -> _ {
        let rows: LookupMany<'a> = lookup_many(keys);
        rows.select::<(raw_state, AsSelect<BestTimes, Mysql>)>((raw_state, BestTimes::as_select()))
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
        let unp: Unprocessed = unprocessed();
        unp.and(queued.eq(false))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn available(max_time: u32) -> _ {
        let p: Preserved = preserved();
        p.and((elapsed + estimated_remaining).lt(max_time))
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn estimated_total() -> _ {
        elapsed + estimated_remaining
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn time_since_and_estimated_total() -> _ {
        let sc: EstimatedTotal = estimated_total();
        (time_since_visit, sc)
    }

    #[auto_type(type_case = "PascalCase")]
    pub fn best_available(n: i64, min_progress: u32, max_time: u32) -> _ {
        let av: Available = available(max_time);
        let sc: EstimatedTotal = estimated_total();
        db_states
            .filter(av.and(progress.ge(min_progress)))
            .order_by((progress, sc))
            .limit(n)
    }

    pub fn full_history() -> SqlQuery {
        // If the first result isn't the starting state, there's a recursive loop in the states
        // that needs to be fixed.
        diesel::sql_query(
            r#"
            WITH RECURSIVE FullHistory(raw_state, prev, hist, elapsed, ri)
            AS (
            -- Anchor definition = the end state
            SELECT raw_state, prev, hist, elapsed, 0 AS ri
                FROM db_states
                WHERE raw_state = ?
            UNION DISTINCT
            -- Recursive definition = the state pointed to by the previous state's prev
            SELECT db.raw_state, db.prev, db.hist, db.elapsed, fh.ri + 1 AS ri
            FROM db_states as db
            INNER JOIN FullHistory as fh
                ON db.raw_state = fh.prev
            )
            SELECT raw_state, prev, hist, elapsed
            FROM FullHistory
            ORDER BY ri DESC
            "#,
        )
    }

    fn downstream() -> SqlQuery {
        // We can support performing downstream updates from multiple states
        // as long as they are not in each other's downstreams
        // because each is the root of its own subtree, so they cannot share any downstream states
        // without being fully contained in another subtree.
        // As long as the states are from the same "next" cohort of a processed state, then if one of them
        // is updated, it's guaranteed to be a root directly under that processed state. And if one is not
        // updated, then it will not generate changes in the recursive part of the query thanks to the WHERE condition
        // even if it is in another downstream.
        // And we leave mysql in autocommit mode to ensure that all statements are transactions
        diesel::sql_query(
            r#"
            WITH RECURSIVE Downstream(raw_state, prev, elapsed, step_time, new_elapsed, new_time_since_visit)
            AS (
            -- Anchor definition = the updated states
            (SELECT raw_state, prev, elapsed, step_time, elapsed AS new_elapsed, time_since_visit as new_time_since_visit
                FROM db_states
                WHERE prev = ?
                FOR UPDATE OF db_states)
            UNION
            -- Recursive definition = the states that point to earlier states via prev
            SELECT db.raw_state, db.prev, db.elapsed, db.step_time, prior.new_elapsed + db.step_time AS new_elapsed,
                IF(db.time_since_visit = 0, 0, prior.new_time_since_visit + db.step_time) AS new_time_since_visit
            FROM db_states AS db
            INNER JOIN Downstream AS prior
                ON db.prev = prior.raw_state
            WHERE prior.new_elapsed + db.step_time < db.elapsed
            FOR UPDATE OF db
            ), SortedDownstream(raw_state, prev, elapsed, step_time, new_elapsed, new_time_since_visit)
            AS (
            SELECT * FROM Downstream
            ORDER BY raw_state
            )
            "#,
        )
    }

    pub fn test_downstream() -> SqlQuery {
        downstream().sql(
            r#"
            SELECT raw_state, elapsed AS old_elapsed, new_elapsed, new_time_since_visit, step_time FROM SortedDownstream
            ORDER BY new_elapsed
            "#,
        )
    }

    pub fn improve_downstream() -> SqlQuery {
        downstream().sql(
            r#"
            -- Perform the update on the states we've selected and paired with improved times
            UPDATE db_states
            INNER JOIN SortedDownstream AS res ON db_states.raw_state = res.raw_state
            SET db_states.elapsed = res.new_elapsed, db_states.time_since_visit = res.new_time_since_visit
            "#,
        )
    }
}
