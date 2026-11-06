//! Putting back in the queue what a bot never began (spec 9.1): after its
//! process ended, and at start for what the last run left behind.

use botloft_core::ids::BotId;
use tracing::warn;

use crate::clock;
use crate::state::{Daemon, Event};

/// Why a delivery goes back in the queue after its process ended.
const PROCESS_ENDED: &str = "the bot's process ended before it began this message";
/// Why a delivery goes back in the queue when the daemon starts.
const DAEMON_STOPPED: &str = "Botloft stopped before the agent began this message";

/// The process of `generation` ended: what it never began goes back in
/// the queue, counting an attempt (spec 9.1).
pub fn requeue_unread(daemon: &Daemon, bot: &BotId, generation: u64) {
    let now = daemon.clock.now_ms();
    let store = daemon.store();
    let unread = match store.unread_deliveries(bot, generation) {
        Ok(unread) => unread,
        Err(err) => {
            warn!(bot = %bot, "could not read unread deliveries: {err}");
            return;
        }
    };
    let settings = &daemon.courier.settings;
    for delivery in unread {
        let attempts = delivery.attempts + 1;
        let retry_at = (attempts < settings.max_attempts)
            .then(|| clock::after(now, settings.retry_delay(attempts)));
        match store.reopen_unread(&delivery.id, PROCESS_ENDED, retry_at, now) {
            Ok(Some(changed)) => {
                if retry_at.is_none() {
                    warn!(delivery = %changed.id, bot = %bot, "gave up on a delivery");
                }
                daemon.emit(Event::DeliveryChanged(changed));
            }
            Ok(None) => {}
            Err(err) => warn!(delivery = %delivery.id, "could not requeue: {err}"),
        }
    }
    drop(store);
    daemon.courier.wake();
}

/// At start, before any bot runs: no process of the last run survived it,
/// and the ones that stopped with the daemon never got to end on their own,
/// so what they were written and never began goes back in the queue now,
/// without counting an attempt (spec 9.1). A routine run waiting on such a
/// message goes on when it is delivered again (spec 20.3).
pub fn requeue_left_behind(daemon: &Daemon) {
    let now = daemon.clock.now_ms();
    let reopened = daemon.store().reopen_left_behind(DAEMON_STOPPED, now);
    match reopened {
        Ok(reopened) => {
            for delivery in reopened {
                daemon.emit(Event::DeliveryChanged(delivery));
            }
        }
        Err(err) => warn!("could not requeue what the last run left unread: {err}"),
    }
    daemon.courier.wake();
}
