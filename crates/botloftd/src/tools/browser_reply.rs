//! What the browser tools answer (spec 21.6): the page as the bot reads it,
//! the action for the owner's cursor, and errors in words the bot can act
//! on.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserAction, BrowserActionKind};

use super::browser::{Bot, Reply};
use super::browser_sites::outside_file;
use crate::browser::{Aim, BrowserError, Done, Session, sites};
use crate::state::Daemon;

/// The page as the bot reads it, with what happened on the way.
pub(super) async fn answer(
    session: &Session,
    bot: &Bot,
    from: usize,
    notes: Option<Vec<String>>,
) -> Result<Reply, String> {
    let reading = session.read(from).await.map_err(|err| describe(&err))?;
    if sites::is_file(&reading.url) {
        outside_file(session, bot, &reading.url).await?;
    }
    let mut out = format!(
        "Page: {}\nURL: {}\n",
        if reading.title.is_empty() {
            "(no title)"
        } else {
            &reading.title
        },
        reading.url
    );
    for note in session.take_notes().iter().chain(notes.iter().flatten()) {
        out.push_str(note);
        out.push('\n');
    }
    out.push('\n');
    if reading.text.is_empty() {
        out.push_str(if from > 0 {
            "(Nothing more.)"
        } else {
            "(The page shows no text.)"
        });
    } else {
        out.push_str(&reading.text);
    }
    let next = from + reading.text.encode_utf16().count();
    if reading.total > next {
        out.push_str(&format!(
            "\n\n[{} more characters: call browser_look with from: {next}]",
            reading.total - next
        ));
    }
    Ok(Reply::Text(out))
}

/// How long the owner's cursor takes to glide to a point (`.bot-cursor` in
/// the app's `motion.css`, 260 ms), with a little to spare. It was 520 ms,
/// which made every click, typing and choice take half a second more while
/// the owner watched (spec 21.7).
const GLIDE: Duration = Duration::from_millis(300);

/// Shows the owner where the bot is about to act and, while someone
/// watches, waits for the cursor to get there, so the page changes after
/// the cursor arrives and not before (spec 21.7). Says whether it waited.
pub(super) async fn point(
    daemon: &Daemon,
    bot: &BotId,
    kind: BrowserActionKind,
    aim: &Aim,
) -> bool {
    report(daemon, bot, kind, &aim.done);
    if daemon.browsers.watched(bot) {
        tokio::time::sleep(GLIDE).await;
        return true;
    }
    false
}

pub(super) fn report(daemon: &Daemon, bot: &BotId, kind: BrowserActionKind, done: &Done) {
    daemon.browsers.action(BrowserAction {
        bot_id: bot.clone(),
        kind,
        x: done.point.map(|(x, _)| x),
        y: done.point.map(|(_, y)| y),
        label: done.label.clone(),
        at: daemon.clock.now_ms(),
    });
}

pub(super) fn describe(err: &BrowserError) -> String {
    match err {
        BrowserError::Io(_) | BrowserError::Cdp(_) => {
            format!("The browser failed: {err}. Try again; if it keeps failing, tell the owner.")
        }
        other => {
            let text = capitalize(&other.to_string());
            if text.ends_with(['.', '!', '?']) {
                text
            } else {
                format!("{text}.")
            }
        }
    }
}

pub(super) fn unavailable(err: &BrowserError) -> String {
    format!(
        "Your browser could not start: {err}. Tell the owner; Botloft uses Microsoft Edge \
         (Google Chrome or Chromium off Windows), or another Chromium browser set in its config."
    )
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
