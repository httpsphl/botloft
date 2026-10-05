//! What the desktop tools share (spec 24.2, 24.3): finding the window a
//! bot names, what is never granted, asking the owner for an app the first
//! time, and the words for what went wrong.

use botloft_core::chat::DESKTOP_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopAction, DesktopGrant, DesktopLevel};
use serde_json::json;

use super::calls::explain;
use super::desktop_list::covering;
use crate::approvals::{self, Answer};
use crate::platform::desktop::{DesktopError, Window, never};
use crate::service::desktop::changed;
use crate::state::{Daemon, Event};

pub(super) fn describe(err: &DesktopError) -> String {
    match err {
        DesktopError::Unavailable => {
            "Using the owner's desktop is not available on this system yet.".to_owned()
        }
        DesktopError::Gone => {
            "That window is not open anymore. Call desktop_windows to see what is open.".to_owned()
        }
        DesktopError::Minimized => "That window is minimized, so there is nothing to see in it. \
            Ask the owner to bring it back if you need it."
            .to_owned(),
        DesktopError::System(why) => format!("Windows could not do it: {why}"),
        DesktopError::NotThere => "That control is not in the window anymore. Call desktop_look \
            to read the window again."
            .to_owned(),
        DesktopError::Cannot(what) => format!(
            "That control cannot be {what} through accessibility. Try another control that does \
             the same, or ask the owner."
        ),
        DesktopError::Password => "That is a password field: the owner types there, never you. \
            Ask them if it needs filling."
            .to_owned(),
        DesktopError::ReadOnly => "That field cannot be changed.".to_owned(),
        DesktopError::NoOption(option) => format!(
            "There is no option \"{option}\" there. Read the window again to see the options."
        ),
        DesktopError::OwnerTookOver => "The owner moved the mouse or pressed a key, so you \
            stopped at once: they are using their computer. Wait, then try again; do not fight \
            them for the mouse."
            .to_owned(),
        DesktopError::NotInFront => "Windows did not bring the app's window to the front, or \
            another window came over it, so nothing was sent. Try again in a moment."
            .to_owned(),
        DesktopError::Covered => "Another window covers that point, so the click was not sent. \
            Read the windows again."
            .to_owned(),
        DesktopError::Locked => "The screen is locked or takes no input now, so the real mouse \
            and keyboard cannot be used. Accessibility may still work."
            .to_owned(),
    }
}

/// Runs a platform call off the async threads: they reach other processes.
pub(super) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, DesktopError> + Send + 'static,
) -> Result<T, String> {
    match tokio::task::spawn_blocking(work).await {
        Ok(result) => result.map_err(|err| describe(&err)),
        Err(_) => Err("Botloft could not reach the desktop.".to_owned()),
    }
}

pub(super) fn heading(window: &Window) -> String {
    format!(
        "Window {}: \"{}\" ({})",
        window.id, window.title, window.app.name
    )
}

/// What the bot may do in an app, as the owner reads it.
fn verb(level: DesktopLevel) -> &'static str {
    match level {
        DesktopLevel::See => "see",
        DesktopLevel::Act => "use",
    }
}

/// The bot used `app` with the owner away: the app tells them when they
/// are back, with the chat where it began.
pub(super) fn used_away(daemon: &Daemon, bot: &BotId, app: &str) {
    let item = daemon
        .store()
        .chat_history(bot, None, 1)
        .ok()
        .and_then(|items| items.into_iter().next())
        .map(|item| item.id);
    let now = daemon.clock.now_ms();
    let uses = daemon.desktop.away.record(bot, app, now, item);
    daemon.emit(Event::DesktopAway(uses));
}

/// The window `id`, once the bot may see or act in its app (`level`):
/// asks the owner the first time, with the bot's `why` (spec 24.2).
#[allow(clippy::too_many_arguments)]
pub(super) async fn granted(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    windows: &[Window],
    grants: &[DesktopGrant],
    id: u64,
    why: Option<&str>,
    level: DesktopLevel,
) -> Result<Window, String> {
    let window = windows
        .iter()
        .find(|window| window.id == id)
        .cloned()
        .ok_or_else(|| describe(&DesktopError::Gone))?;
    if let Some(never) = never(&window) {
        return Err(format!(
            "You may never use that window: {}. Ask the owner if you need something there.",
            never.why()
        ));
    }
    // Without the owner there, only a grant for use while they are away
    // reaches the window, and nobody is asked (spec 24.8).
    if let Err(away) = daemon.desktop.owner_here() {
        if !covering(grants, &window, level).is_some_and(|grant| grant.unattended) {
            return Err(away.why().to_owned());
        }
        used_away(daemon, bot, &window.app.name);
        return Ok(window);
    }
    if covering(grants, &window, level).is_some() {
        return Ok(window);
    }
    let app = &window.app;
    let Some(why) = why else {
        return Err(format!(
            "The owner has not let you {} {} yet. Call the tool again with why: one short \
             sentence on what you need it for, and they are asked.",
            verb(level),
            app.name
        ));
    };
    let path = app.path.to_string_lossy().into_owned();
    let input = json!({
        "app": app.name,
        "path": path,
        "level": level.as_str(),
        "why": why,
    });
    match approvals::ask(daemon, bot, generation, DESKTOP_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            daemon
                .store()
                .grant_desktop_app(bot, &path, &app.name, level, daemon.clock.now_ms())
                .map_err(|err| explain(err.into()))?;
            changed(daemon, bot).map_err(explain)?;
            Ok(window)
        }
        Some(Answer::Denied { note }) => {
            let said = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Err(format!(
                "The owner did not let you {} {}.{said} Do without it or ask them.",
                verb(level),
                app.name
            ))
        }
        Some(Answer::Expired) => Err(format!(
            "The owner did not answer about {} in time. Try again later or do without it.",
            app.name
        )),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

/// The bot read or acted in `window`: its panel shows it (spec 24.9).
pub(super) fn used(daemon: &Daemon, bot: &BotId, window: &Window, action: Option<DesktopAction>) {
    let now = daemon.clock.now_ms();
    let state = daemon.desktop.activity.used(bot, window, action, now);
    daemon.emit(Event::DesktopChanged(state));
}
