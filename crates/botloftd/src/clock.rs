//! Wall-clock time for everything the daemon stores or compares with stored
//! times: delivery retries, leases and deadlines (spec 9.1). Tests move a
//! [`ManualClock`] by hand instead of waiting.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

pub trait Clock: Send + Sync + 'static {
    /// Unix time in milliseconds.
    fn now_ms(&self) -> i64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        botloft_core::now_ms()
    }
}

/// Time that only moves when told to.
#[derive(Debug)]
pub struct ManualClock {
    now: AtomicI64,
}

impl ManualClock {
    /// Starts at the current time, so stored values look real.
    pub fn new() -> Self {
        Self {
            now: AtomicI64::new(botloft_core::now_ms()),
        }
    }

    pub fn advance(&self, by: Duration) {
        let ms = i64::try_from(by.as_millis()).unwrap_or(i64::MAX);
        self.now.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
}

/// `at` plus `by`, in Unix milliseconds.
pub fn after(at: i64, by: Duration) -> i64 {
    at.saturating_add(i64::try_from(by.as_millis()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manual_clock_moves_only_when_advanced() {
        let clock = ManualClock::new();
        let start = clock.now_ms();
        assert_eq!(clock.now_ms(), start);
        clock.advance(Duration::from_secs(5));
        assert_eq!(clock.now_ms(), start + 5_000);
        assert_eq!(after(start, Duration::from_millis(250)), start + 250);
    }
}
