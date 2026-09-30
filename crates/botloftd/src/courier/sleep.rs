//! How long the courier sleeps between cycles, and catching up with a bot
//! that became ready (spec 9.1).

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use tokio::sync::broadcast;
use tracing::warn;

use crate::state::{Daemon, Event};

/// Shortest sleep between cycles, so a delivery that stays due (its
/// prepare failed) cannot spin the courier.
const MIN_WAIT: Duration = Duration::from_millis(250);

/// How long to sleep before the next cycle.
pub(super) fn next_wait(daemon: &Daemon) -> Duration {
    let most = daemon.courier.settings.poll_interval;
    let next = match daemon.store().courier_next_at() {
        Ok(next) => next,
        Err(err) => {
            warn!("courier could not plan its next cycle: {err}");
            None
        }
    };
    let Some(at) = next else {
        return most;
    };
    let until = at.saturating_sub(daemon.clock.now_ms()).max(0);
    Duration::from_millis(u64::try_from(until).unwrap_or(0)).clamp(MIN_WAIT.min(most), most)
}

/// The next bot that can take messages again (spec 9.1). `None` when
/// changes were missed: a cycle catches up with all of them.
pub(super) async fn became_ready(states: &mut broadcast::Receiver<Event>) -> Option<BotId> {
    loop {
        match states.recv().await {
            Ok(Event::BotState(change))
                if matches!(
                    change.state,
                    BotState::Idle | BotState::Busy | BotState::NeedsApproval
                ) =>
            {
                return Some(change.bot_id);
            }
            Ok(_) => {}
            Err(broadcast::error::RecvError::Lagged(_)) => return None,
            Err(broadcast::error::RecvError::Closed) => std::future::pending().await,
        }
    }
}

/// What waited only for `bot` to be ready goes in the next cycle, not at
/// its next check.
pub(super) fn hasten(daemon: &Daemon, bot: &BotId) {
    let now = daemon.clock.now_ms();
    if let Err(err) = daemon.store().hasten_deliveries(bot, now) {
        warn!(bot = %bot, "courier could not bring deliveries forward: {err}");
    }
}
