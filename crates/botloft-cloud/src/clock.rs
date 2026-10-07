//! The time the server goes by, in ms Unix. Tests move it forward instead of
//! waiting for a link to expire.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Default)]
pub struct Clock(Arc<AtomicI64>);

impl Clock {
    pub fn now(&self) -> i64 {
        let real = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_millis() as i64);
        real + self.0.load(Ordering::Relaxed)
    }

    /// Jumps `ms` ahead, for tests.
    pub fn advance(&self, ms: i64) {
        self.0.fetch_add(ms, Ordering::Relaxed);
    }
}
