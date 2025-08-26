use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
