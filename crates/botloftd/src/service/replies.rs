//! Where an answer goes (spec 40.2): back to the conversation that asked.
//! A task and an agent's own routine or reminder keep the conversation
//! they came from; what shows in the chat (a question, a card, a key
//! request, an approval) is in it. Anything else goes to the main one, and
//! so does an answer whose conversation was archived since (the courier
//! moves it).

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::{Routine, Task};
use tracing::warn;

use crate::seat::Seat;
use crate::state::Daemon;

/// A routine or reminder an agent made for itself fires in the
/// conversation that made it. One for another agent goes to that agent's
/// main conversation, as the owner's do.
pub(crate) fn keep_routine(daemon: &Daemon, seat: &Seat, routine: &Routine) {
    if seat.is_main() || routine.bot_id != seat.bot {
        return;
    }
    let kept = daemon
        .store()
        .set_routine_conversation(&routine.id, &seat.conversation);
    if let Err(err) = kept {
        warn!(bot = %seat, "could not keep where a routine came from: {err}");
    }
}

/// A task given from one conversation brings its result back there.
pub(crate) fn keep_task(daemon: &Daemon, seat: &Seat, task: &Task) {
    if seat.is_main() {
        return;
    }
    let kept = daemon
        .store()
        .set_task_conversation(&task.id, &seat.conversation);
    if let Err(err) = kept {
        warn!(bot = %seat, "could not keep where a task came from: {err}");
    }
}

/// The conversation found for an answer to `bot`, or its main one.
pub(crate) fn or_main(
    found: botloft_store::Result<Option<ConversationId>>,
    bot: &BotId,
) -> ConversationId {
    found
        .unwrap_or_else(|err| {
            warn!(bot = %bot, "could not read where an answer goes: {err}");
            None
        })
        .unwrap_or_else(|| ConversationId::main_of(bot))
}
