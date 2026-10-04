//! The desktop tools that act (spec 24.5, 24.6, 24.7): a click, typing, a
//! choice or a scroll on a control of the bot's last reading, through
//! accessibility, after the owner let the bot use that app; what
//! accessibility cannot do there, with the real mouse and keyboard where
//! the owner turned them on. Each answers with the window as it reads
//! afterwards.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopAction, DesktopActionKind, DesktopGrant, DesktopLevel};

use super::desktop::used;
use super::desktop_grant::{blocking, describe, granted, heading};
use super::desktop_real::{enter, instead, needs_real, real_allowed};
use crate::desktop::STOPPED;
use crate::platform::desktop::{
    self as platform, Acted, Action, Control, DesktopError, Window, by_reference, render,
};
use crate::state::Daemon;

/// How long the window gets to show what the action did before it is read.
const SETTLE: Duration = Duration::from_millis(400);

/// What the bot asked, on which control of its last reading.
pub(super) struct Ask {
    pub(super) reference: String,
    pub(super) action: Action,
    pub(super) why: Option<String>,
    /// Enter after typing, with the real keyboard.
    pub(super) submit: bool,
}

fn done(action: &Action, control: &Control) -> String {
    let what = if control.name.is_empty() {
        control.kind.to_owned()
    } else {
        format!("{} \"{}\"", control.kind, control.name)
    };
    match action {
        Action::Click => format!("Clicked {what}."),
        Action::Type(_) => format!("Typed in {what}."),
        Action::Select(option) => format!("Chose \"{option}\" in {what}."),
        Action::Scroll(_) => format!("Scrolled {what}."),
    }
}

/// The action as the bot's panel shows it.
fn shown(action: &Action, control: &Control) -> DesktopAction {
    let (kind, option) = match action {
        Action::Click => (DesktopActionKind::Click, None),
        Action::Type(_) => (DesktopActionKind::Type, None),
        Action::Select(option) => (DesktopActionKind::Select, Some(option.clone())),
        Action::Scroll(_) => (DesktopActionKind::Scroll, None),
    };
    DesktopAction {
        kind,
        target: control.name.clone(),
        option,
    }
}

/// The window read again after an action, with `said` first: kept as the
/// bot's reading and shown on its panel.
pub(super) async fn answered(
    daemon: &Daemon,
    bot: &BotId,
    window: Window,
    said: String,
    action: DesktopAction,
) -> Result<String, String> {
    tokio::time::sleep(SETTLE).await;
    let id = window.id;
    let controls = blocking(move || platform::read(id)).await?;
    // Its title may have changed too.
    let window = blocking(platform::windows)
        .await?
        .into_iter()
        .find(|after| after.id == id)
        .unwrap_or(window);
    let (text, next) = render(&controls, 0);
    daemon.desktop.keep(bot, id, controls);
    used(daemon, bot, &window, Some(action));
    let mut answer = format!("{said}\n{}\n{text}", heading(&window));
    if let Some(next) = next {
        answer.push_str(&format!(
            "\nThere is more: call desktop_look with from: {next}."
        ));
    }
    Ok(answer)
}

pub(super) async fn act(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    windows: &[Window],
    grants: &[DesktopGrant],
    ask: Ask,
) -> Result<String, String> {
    let reading = daemon
        .desktop
        .reading(bot)
        .ok_or("Read the window first with desktop_look: the refs come from your last reading.")?;
    let control = by_reference(&reading.controls, &ask.reference)
        .cloned()
        .ok_or_else(|| {
            format!(
                "{} is not in your last reading (window {}). Call desktop_look to read it again.",
                ask.reference, reading.window
            )
        })?;
    if control.password && matches!(ask.action, Action::Type(_)) {
        return Err(describe(&DesktopError::Password));
    }
    let window = granted(
        daemon,
        bot,
        generation,
        windows,
        grants,
        reading.window,
        ask.why.as_deref(),
        DesktopLevel::Act,
    )
    .await?;
    let real = real_allowed(grants, &window);
    if ask.submit && !real {
        return Err(needs_real(&window.app.name));
    }
    let tried = {
        let _turn = daemon.desktop.turn().await;
        // The owner may have stopped it while it waited.
        if daemon.desktop.activity.stopped(bot) {
            return Err(STOPPED.to_owned());
        }
        let (id, target, action) = (window.id, control.runtime_id.clone(), ask.action.clone());
        tokio::task::spawn_blocking(move || platform::act(id, &target, &action))
            .await
            .map_err(|_| "Botloft could not reach the desktop.".to_owned())?
    };
    let mut said = done(&ask.action, &control);
    match tried {
        Ok(Acted::Done) => {}
        Ok(Acted::Waiting) => said.push_str(
            " The app did not answer in time: it may be showing a dialog. Call desktop_windows \
             to see.",
        ),
        // What accessibility cannot do, the real mouse and keyboard may.
        Err(DesktopError::Cannot(what))
            if matches!(ask.action, Action::Click | Action::Type(_)) =>
        {
            if !real {
                return Err(format!(
                    "{} {}",
                    describe(&DesktopError::Cannot(what)),
                    needs_real(&window.app.name)
                ));
            }
            let text = match &ask.action {
                Action::Type(text) => Some(text.clone()),
                _ => None,
            };
            instead(daemon, bot, &window, control.rect, text).await?;
            said.push_str(" With the real mouse and keyboard.");
        }
        Err(err) => return Err(describe(&err)),
    }
    if ask.submit {
        enter(daemon, bot, &window).await?;
        said.push_str(" Pressed Enter.");
    }
    let action = shown(&ask.action, &control);
    answered(daemon, bot, window, said, action).await
}
