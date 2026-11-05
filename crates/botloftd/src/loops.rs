//! The daemon's long-running loops: the supervisor, the courier, the
//! routines and the rest each run for the daemon's whole life. One that ends
//! (a panic, or a return nobody expected) leaves its work undone while the
//! daemon still answers, so the daemon stops with an error instead and the
//! service starts it again (spec 14).

use std::collections::HashMap;
use std::fmt;
use std::future::Future;

use tokio::task::{Id, JoinSet};

#[derive(Default)]
pub struct Loops {
    set: JoinSet<()>,
    names: HashMap<Id, &'static str>,
}

/// Which loop ended, and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopEnded {
    pub name: &'static str,
    pub panicked: bool,
}

impl fmt::Display for LoopEnded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let how = if self.panicked { "panicked" } else { "ended" };
        write!(f, "the {} loop {how}", self.name)
    }
}

impl Loops {
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs `task` as the loop called `name`.
    pub fn spawn(&mut self, name: &'static str, task: impl Future<Output = ()> + Send + 'static) {
        let handle = self.set.spawn(task);
        self.names.insert(handle.id(), name);
    }

    /// Waits for the first loop to end. Never resolves while all of them run,
    /// nor when there are none.
    pub async fn first_ended(&mut self) -> LoopEnded {
        let ended = match self.set.join_next_with_id().await {
            Some(Ok((id, ()))) => (id, false),
            Some(Err(err)) => (err.id(), err.is_panic()),
            None => std::future::pending().await,
        };
        LoopEnded {
            name: self.names.get(&ended.0).copied().unwrap_or("unnamed"),
            panicked: ended.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn a_loop_that_panics_or_returns_is_named() {
        let mut loops = Loops::new();
        loops.spawn("forever", std::future::pending());
        loops.spawn("courier", async { panic!("a bug") });
        assert_eq!(
            loops.first_ended().await,
            LoopEnded {
                name: "courier",
                panicked: true
            }
        );
        loops.spawn("routines", async {});
        let ended = loops.first_ended().await;
        assert_eq!(ended.to_string(), "the routines loop ended");
        let still = tokio::time::timeout(Duration::from_millis(50), loops.first_ended()).await;
        assert!(still.is_err(), "the one still running is not reported");
    }
}
