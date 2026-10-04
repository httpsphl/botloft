//! The desktop tools that act (spec 24.5, 24.6): a click, typing, a choice
//! or a scroll on a control of the bot's last reading, through
//! accessibility, after the owner let the bot use that app. Each answers
//! with the window as it reads afterwards.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopGrant, DesktopLevel};

use super::desktop_grant::{blocking, describe, granted, heading};
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
    let _turn = daemon.desktop.turn().await;
    let (id, target, action) = (window.id, control.runtime_id.clone(), ask.action.clone());
    let acted = blocking(move || platform::act(id, &target, &action)).await?;
    tokio::time::sleep(SETTLE).await;
    let controls = blocking(move || platform::read(id)).await?;
    // Its title may have changed too.
    let window = blocking(platform::windows)
        .await?
        .into_iter()
        .find(|after| after.id == id)
        .unwrap_or(window);
    let (text, next) = render(&controls, 0);
    daemon.desktop.keep(bot, id, controls);
    let mut answer = done(&ask.action, &control);
    if acted == Acted::Waiting {
        answer.push_str(
            " The app did not answer in time: it may be showing a dialog. Call desktop_windows \
             to see.",
        );
    }
    answer.push_str(&format!("\n{}\n{text}", heading(&window)));
    if let Some(next) = next {
        answer.push_str(&format!(
            "\nThere is more: call desktop_look with from: {next}."
        ));
    }
    Ok(answer)
}
