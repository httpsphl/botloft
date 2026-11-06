//! Forgetting the history older than the owner keeps (spec 12.1). Nothing
//! goes unless the owner picked a number of days in Settings. A pass runs a
//! little after start, every few hours and as soon as the setting changes,
//! in small batches so the bots and the app keep the store in between.

use std::sync::Arc;
use std::time::Duration;

use botloft_store::Pruned;
use tokio::sync::Notify;
use tracing::{info, warn};

use crate::state::Daemon;

/// The first pass waits for the start to settle.
const FIRST_AFTER: Duration = Duration::from_secs(2 * 60);
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);
/// Rows of each kind per transaction.
const BATCH: usize = 500;
/// Between two batches, for whoever waits on the store.
const BREATHE: Duration = Duration::from_millis(20);
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Wakes the loop when the owner changes how long the history stays.
static WAKE: Notify = Notify::const_new();

pub fn wake() {
    WAKE.notify_one();
}

/// Prunes for as long as the daemon runs.
pub async fn run(daemon: Arc<Daemon>) {
    tokio::select! {
        () = tokio::time::sleep(FIRST_AFTER) => {}
        () = WAKE.notified() => {}
    }
    loop {
        let pass = Arc::clone(&daemon);
        let _ = tokio::task::spawn_blocking(move || prune(&pass)).await;
        tokio::select! {
            () = tokio::time::sleep(EVERY) => {}
            () = WAKE.notified() => {}
        }
    }
}

/// One pass: everything older than the days kept, batch after batch.
pub fn prune(daemon: &Daemon) -> Pruned {
    let mut total = Pruned::default();
    let Some(days) = daemon.settings.history_days() else {
        return total;
    };
    let before = daemon
        .clock
        .now_ms()
        .saturating_sub(i64::from(days).saturating_mul(DAY_MS));
    loop {
        let pruned = daemon.store().prune_history(before, BATCH);
        match pruned {
            Ok(pruned) if pruned.is_empty() => break,
            Ok(pruned) => total += pruned,
            Err(err) => {
                warn!("could not forget the old history: {err}");
                break;
            }
        }
        std::thread::sleep(BREATHE);
    }
    if !total.is_empty() {
        info!(
            days,
            chat_items = total.chat_items,
            messages = total.messages,
            routine_runs = total.routine_runs,
            "forgot the history older than the days kept"
        );
    }
    total
}
