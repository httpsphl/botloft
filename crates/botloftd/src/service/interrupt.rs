//! `bots.interrupt`: the owner stops what a bot is doing (spec 9.2). Claude
//! Code ends the turn that runs, kills the tool it waits on and the subagents
//! it left in the background, and closes the turn with a `result` that the
//! chat shows as stopped (spec 8.2). Messages it has not taken up yet stay in
//! its queue and run next. Each conversation of a bot stops on its own
//! (spec 40.5).

use botloft_core::protocol::{ApprovalsAnswerParams, Bot, BotConversationParams, BotState};
use tracing::{debug, warn};

use crate::approvals;
use crate::chat::control;
use crate::service::bots::{active, to_protocol};
use crate::service::conversations::seat_of;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

pub fn interrupt(daemon: &Daemon, params: BotConversationParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (_, record) = active(&store, &params.bot_id)?;
    let seat = seat_of(&store, &record, params.conversation_id, true)?;
    // `approvals::answer` takes the store again.
    drop(store);
    let agent = crate::agent::of(record.agent);
    if !agent.is_some_and(|agent| agent.can_interrupt()) {
        return Err(ApiError::validation(format!(
            "bots that run on {} cannot be stopped in the middle of a turn",
            record.agent.as_str()
        )));
    }
    let working = matches!(
        daemon.supervisor.status_of(&seat.conversation),
        Some((BotState::Busy | BotState::NeedsApproval, _))
    );
    // A turn that ended by itself a moment ago leaves nothing to stop.
    if working {
        daemon
            .supervisor
            .write_control_to(&seat.conversation, control::interrupt_request())
            .map_err(|_| ApiError::Conflict(format!("bot {} is not running", record.id)))?;
        debug!(bot = %record.id, "interrupt asked");
        deny_open_requests(daemon, &seat);
    }
    let store = daemon.store();
    let (crew, record) = active(&store, &params.bot_id)?;
    Ok(to_protocol(daemon, &store, &crew, record))
}

/// A tool of that conversation that waits for the owner is refused: the
/// owner said no to all of it, and the card shows so instead of waiting
/// until it expires. Other conversations keep theirs (spec 40.4).
fn deny_open_requests(daemon: &Daemon, bot: &crate::seat::Seat) {
    let open = match daemon.store().pending_approvals_in(&bot.conversation) {
        Ok(open) => open,
        Err(err) => {
            warn!(bot = %bot, "could not read the open requests: {err}");
            return;
        }
    };
    for approval_id in open {
        let params = ApprovalsAnswerParams {
            approval_id,
            allow: false,
            note: None,
            input: None,
            always: None,
            once: None,
        };
        // One that was just answered or expired is already closed.
        let _ = approvals::answer(daemon, params);
    }
}
