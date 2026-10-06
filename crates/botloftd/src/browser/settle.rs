//! Waiting for the page after an action (spec 21.3): a navigation the
//! action started loads, then the network goes quiet, each with a limit.

use std::time::{Duration, Instant};

use super::session::Session;

const LOAD_WAIT: Duration = Duration::from_secs(15);
const QUIET: Duration = Duration::from_millis(500);
const QUIET_WAIT: Duration = Duration::from_secs(3);
/// The same wait on a page that was never quiet before the action (an
/// analytics script, a chat widget, a live feed): its network will not go
/// quiet because of the click, so waiting for it only costs the bot 3 s per
/// action (measured: 3.2 s a click, against 0.17 s on a quiet page).
const NOISY_WAIT: Duration = Duration::from_millis(600);

/// The active tab before an action, to tell whether it navigated.
pub(super) struct Marks {
    target: String,
    navigations: u64,
    /// The network was busy just before the action, whatever the action does.
    noisy: bool,
}

impl Session {
    pub(super) fn marks(&self) -> Marks {
        let tabs = self.lock();
        let tab = tabs.active();
        Marks {
            target: tab.map(|tab| tab.target.clone()).unwrap_or_default(),
            navigations: tab.map_or(0, |tab| tab.navigations),
            noisy: tab
                .is_some_and(|tab| !tab.inflight.is_empty() || tab.network_at.elapsed() < QUIET),
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
        let deadline = Instant::now() + if before.noisy { NOISY_WAIT } else { QUIET_WAIT };
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
