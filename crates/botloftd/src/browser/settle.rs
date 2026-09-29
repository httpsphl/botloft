//! Waiting for the page after an action (spec 21.3): a navigation the
//! action started loads, then the network goes quiet, each with a limit.

use std::time::{Duration, Instant};

use super::session::Session;

const LOAD_WAIT: Duration = Duration::from_secs(15);
const QUIET: Duration = Duration::from_millis(500);
const QUIET_WAIT: Duration = Duration::from_secs(3);

/// The active tab before an action, to tell whether it navigated.
pub(super) struct Marks {
    target: String,
    navigations: u64,
}

impl Session {
    pub(super) fn marks(&self) -> Marks {
        let tabs = self.lock();
        let tab = tabs.active();
        Marks {
            target: tab.map(|tab| tab.target.clone()).unwrap_or_default(),
            navigations: tab.map_or(0, |tab| tab.navigations),
        }
    }

    /// Waits for a navigation the action started to load, then for the
    /// network to go quiet, each with a limit (spec 21.3).
    pub(super) async fn settle(&self, before: Marks) {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let deadline = Instant::now() + LOAD_WAIT;
        loop {
            let notified = self.changed.notified();
            let loading = {
                let tabs = self.lock();
                !tabs.closed
                    && tabs.active().is_some_and(|tab| {
                        (tab.target != before.target || tab.navigations > before.navigations)
                            && tab.loading
                    })
            };
            let left = deadline.saturating_duration_since(Instant::now());
            if !loading || left.is_zero() {
                break;
            }
            let _ = tokio::time::timeout(left, notified).await;
        }
        let deadline = Instant::now() + QUIET_WAIT;
        loop {
            let notified = self.changed.notified();
            let quiet_for = {
                let tabs = self.lock();
                match tabs.active() {
                    Some(tab) if !tabs.closed && !tab.inflight.is_empty() => None,
                    Some(tab) if !tabs.closed => Some(tab.network_at.elapsed()),
                    _ => return,
                }
            };
            let left = deadline.saturating_duration_since(Instant::now());
            if quiet_for.is_some_and(|quiet| quiet >= QUIET) || left.is_zero() {
                return;
            }
            let wait = quiet_for.map_or(Duration::from_millis(100), |quiet| QUIET - quiet);
            let _ = tokio::time::timeout(wait.min(left), notified).await;
        }
    }
}
