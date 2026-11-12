//! How long the courier sleeps between cycles, and catching up with a bot
//! that became ready (spec 9.1).

use std::time::Duration;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::{BotState, Delivery};
use botloft_store::{Store, StoreError};
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
            // Each conversation, the main one too: the bot's own state is
            // a summary that may not change when one becomes ready.
            Ok(Event::SeatState(change))
                if matches!(
                    change.state,
                    BotState::Idle | BotState::Busy | BotState::NeedsApproval
                ) =>
            {
                return Some(change.seat.bot);
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

/// What waited for a conversation the owner archived goes to the main one
/// (spec 40.2).
pub(super) fn rehome(store: &Store, due: Vec<Delivery>) -> Vec<Delivery> {
    due.into_iter()
        .map(|mut delivery| {
            if let Err(err) = to_main_if_archived(store, &mut delivery) {
                warn!(delivery = %delivery.id, "courier could not move a delivery: {err}");
            }
            delivery
        })
        .collect()
}

fn to_main_if_archived(store: &Store, delivery: &mut Delivery) -> Result<(), StoreError> {
    let conversation = &delivery.conversation_id;
    if conversation.is_main_of(&delivery.bot_id) {
        return Ok(());
    }
    if store
        .conversation(conversation)?
        .is_some_and(|found| found.archived_at.is_none())
    {
        return Ok(());
    }
    let main = ConversationId::main_of(&delivery.bot_id);
    store.move_delivery(&delivery.id, &main)?;
    delivery.conversation_id = main;
    Ok(())
}
