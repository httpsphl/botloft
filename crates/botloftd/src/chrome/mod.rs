//! The owner's own Chrome in the agents (spec 37): the agents the owner
//! gave it to, how they start, who acts in it now, and the requests of its
//! tools (the Claude in Chrome extension, `mcp__claude-in-chrome__*`).

mod prompt;
mod rules;
pub(crate) mod seen;
mod turn;

use std::io;
use std::path::Path;

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{
    BotChrome, ChromeSetParams, ChromeTurn, Message, MessageKind, SenderKind,
};
use tracing::{info, warn};

pub use self::prompt::prompt;
pub(crate) use self::prompt::{Kind, RULE_TOOL, kind_of};
use crate::service::bots::active;
use crate::service::{ApiError, ApiResult};
use crate::state::{Daemon, Event};

/// What the daemon keeps about the owner's Chrome while it runs.
#[derive(Default)]
pub struct Chrome {
    /// The site of each tab an agent opened (spec 37.3).
    tabs: prompt::Tabs,
    /// The agent acting in it now (spec 37.4).
    turn: turn::Turn,
}

/// Whether the owner gave `bot` their Chrome.
pub fn given(daemon: &Daemon, bot: &BotId) -> bool {
    daemon.store().bot_chrome(bot).ok().flatten().is_some()
}

/// Whether `bot` starts with `--chrome` (spec 37.2), with its rules
/// written. Only agents on Claude Code get it. Without the Chrome, the
/// tabs it knew are forgotten; with it, another conversation's process
/// may still use them, and every answer of the Chrome says where they
/// are (`seen`).
pub(crate) fn launch(daemon: &Daemon, bot: &BotId, workspace: &Path) -> io::Result<bool> {
    let on_claude = crate::connectors::on_claude(daemon, bot);
    let chrome = on_claude && given(daemon, bot);
    if !chrome {
        daemon.chrome.tabs.forget(bot);
    }
    rules::write(workspace, on_claude, chrome)?;
    Ok(chrome)
}

/// `chrome.set`: gives the owner's Chrome to an agent, once the owner
/// accepted the risks, or takes it back. The agent starts again, with
/// `--resume`, once nothing is under way.
pub fn set(daemon: &Daemon, params: ChromeSetParams) -> ApiResult<BotChrome> {
    crate::service::bots::find(&daemon.store(), &params.bot_id)?;
    if params.on {
        if !crate::connectors::on_claude(daemon, &params.bot_id) {
            return Err(ApiError::Conflict(
                "only agents on Claude Code can use the owner's Chrome".to_owned(),
            ));
        }
        if !params.accepted_risks {
            return Err(ApiError::Conflict(
                "the owner has to accept the risks first".to_owned(),
            ));
        }
        daemon
            .store()
            .give_bot_chrome(&params.bot_id, daemon.clock.now_ms())?;
        info!("the owner gave their Chrome to an agent");
    } else if daemon.store().take_bot_chrome(&params.bot_id)? {
        turn::release(daemon, &params.bot_id);
        info!("the owner took their Chrome back from an agent");
    }
    changed(daemon, &params.bot_id)
}

/// Gives the Chrome the agent asked for with `ask_chrome` and queues a note
/// that tells it to go on once it has started again (spec 37.5).
pub(crate) fn grant(daemon: &Daemon, bot: &BotId) -> ApiResult<()> {
    set(
        daemon,
        ChromeSetParams {
            bot_id: bot.clone(),
            on: true,
            accepted_risks: true,
        },
    )?;
    let store = daemon.store();
    let (crew, _) = active(&store, bot)?;
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.clone(),
        kind: MessageKind::System,
        body: "The owner gave you their Chrome. Its tools (mcp__claude-in-chrome__*) are \
               ready now: go on with what you were doing."
            .to_owned(),
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = crate::service::messages::post(daemon, &store, note, None) {
        warn!(bot = %bot, "could not tell an agent about the Chrome: {err}");
    }
    Ok(())
}

fn changed(daemon: &Daemon, bot: &BotId) -> ApiResult<BotChrome> {
    let chrome = BotChrome {
        bot_id: bot.clone(),
        given_at: daemon.store().bot_chrome(bot)?,
    };
    daemon.emit(Event::BotChrome(chrome.clone()));
    daemon.supervisor.launch_settings_changed(bot);
    Ok(chrome)
}

/// `chrome.status`: the agent acting in the owner's Chrome now.
pub fn status(daemon: &Daemon) -> ChromeTurn {
    ChromeTurn {
        bot_id: daemon.chrome.turn.holder(),
    }
}

/// `chrome.stop`: the agent acting in the Chrome lets go of it, and its
/// next actions there are refused until its turn ends (spec 37.4).
pub fn stop(daemon: &Daemon) -> ChromeTurn {
    if let Some(bot) = daemon.chrome.turn.holder() {
        daemon.chrome.turn.stop(&bot);
        turn::release(daemon, &bot);
        info!("the owner stopped an agent in their Chrome");
    }
    status(daemon)
}

/// The agent's turn ended: it lets go of the Chrome, and a stop the owner
/// gave no longer holds.
pub(crate) fn turn_ended(daemon: &Daemon, bot: &BotId) {
    daemon.chrome.turn.forget_stop(bot);
    turn::release(daemon, bot);
}

/// Turns may have ended unseen: nobody keeps the Chrome.
pub(crate) fn turns_lost(daemon: &Daemon) {
    daemon.chrome.turn.forget_stops();
    if let Some(bot) = daemon.chrome.turn.holder() {
        turn::release(daemon, &bot);
    }
}
