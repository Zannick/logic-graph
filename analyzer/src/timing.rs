use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LockResult, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct WallTimeStats {
    total_micros: AtomicUsize,
    num_calls: AtomicUsize,
}

impl WallTimeStats {
    pub fn record(&self, dur: Duration) {
        self.total_micros
            .fetch_add(dur.as_micros() as usize, Ordering::Acquire);
        self.num_calls.fetch_add(1, Ordering::Release);
    }

    pub fn average(&self) -> Duration {
        let total_micros = self.total_micros.load(Ordering::Acquire);
        let num_calls = self.num_calls.load(Ordering::Acquire);
        if num_calls == 0 {
            Duration::default()
        } else {
            Duration::from_micros(total_micros as u64) / num_calls as u32
        }
    }
}

impl std::fmt::Debug for WallTimeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_micros = self.total_micros.load(Ordering::Acquire);
        let num_calls = self.num_calls.load(Ordering::Acquire);
        if num_calls == 0 {
            f.write_str("No data")
        } else {
            let avg = Duration::from_micros(total_micros as u64) / num_calls as u32;
            f.write_fmt(format_args!("avg: {:?} ({} calls)", avg, num_calls))
        }
    }
}

pub struct TimingMutex<T: ?Sized> {
    mutex: Mutex<T>,
}

impl<T: ?Sized> Deref for TimingMutex<T> {
    type Target = Mutex<T>;
    fn deref(&self) -> &Self::Target {
        &self.mutex
    }
}

impl<T: ?Sized> DerefMut for TimingMutex<T> {
    fn deref_mut(&mut self) -> &mut Mutex<T> {
        &mut self.mutex
    }
}

unsafe impl<T: ?Sized + Send> Send for TimingMutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for TimingMutex<T> {}

impl<T: Sized> TimingMutex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            mutex: Mutex::new(inner),
        }
    }

    pub fn into_inner(self) -> LockResult<T> {
        self.mutex.into_inner()
    }

    pub fn track_lock<'a>(&'a self, stats: &'a WallTimeStats) -> LockResult<TimingMutexGuard<'a, T>> {
        match self.mutex.lock() {
            Ok(g) => Ok(TimingMutexGuard::new(g, stats)),
            Err(e) => Err(PoisonError::new(TimingMutexGuard::new(
                e.into_inner(),
                stats,
            ))),
        }
    }
}

pub struct TimingMutexGuard<'a, T: ?Sized + 'a> {
    guard: MutexGuard<'a, T>,
    stats: &'a WallTimeStats,
    acq: Instant,
}

impl<'a, T: ?Sized + 'a> Deref for TimingMutexGuard<'a, T> {
    type Target = MutexGuard<'a, T>;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T: ?Sized + 'a> DerefMut for TimingMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T: ?Sized + 'a> Drop for TimingMutexGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        self.stats.record(self.acq.elapsed());
    }
}

impl<'a, T: ?Sized + 'a> TimingMutexGuard<'a, T> {
    fn new(guard: MutexGuard<'a, T>, stats: &'a WallTimeStats) -> Self {
        Self {
            guard,
            stats,
            acq: Instant::now(),
        }
    }
}
