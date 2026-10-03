//! The browser tools (spec 21.4): each acts in the bot's own browser, asks
//! the owner before a new site when the bot's mode asks first (spec 21.5),
//! waits while the owner has it and reads first if they left another tab
//! open (spec 21.10), and answers with the page as it is after the action.

use std::path::PathBuf;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserActionKind, PermissionMode};
use serde_json::{Value, json};
use tracing::debug;

use super::browser_args::Tool;
use super::browser_help::ask_owner;
use super::browser_reply::{answer, describe, point, report, unavailable};
use super::browser_sites::{allowed, page_allowed};
use super::calls::explain;
use crate::browser::{Done, OWNER_WAIT, Scroll, sites};
use crate::service::bots;
use crate::state::Daemon;

/// What the bot reads when the owner gave its browser back on another tab
/// than the one it was on (spec 21.10).
const SWITCHED: &str =
    "The owner switched tabs while they had your browser. This is the tab that is open now.";
const NOT_DONE: &str = "The owner switched tabs while they had your browser, so this was not \
    done: the page you were on is not the one open now. Call browser_look to read the tab that \
    is open, then decide what to do.";

/// What a browser tool answers.
pub(super) enum Reply {
    Text(String),
    /// A JPEG of the screen and a line about the page.
    Picture(String, String),
}

/// Where the bot's files are and how it asks.
pub(super) struct Bot {
    pub(super) mode: PermissionMode,
    pub(super) workspace: PathBuf,
    pub(super) folders: Vec<PathBuf>,
}

pub(super) async fn call(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    name: &str,
    arguments: Value,
) -> Value {
    debug!(bot = %bot, tool = name, "browser tool");
    match run(daemon, bot, generation, name, arguments).await {
        Ok(Reply::Text(text)) => json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false,
        }),
        Ok(Reply::Picture(data, text)) => json!({
            "content": [
                { "type": "image", "data": data, "mimeType": "image/jpeg" },
                { "type": "text", "text": text },
            ],
            "isError": false,
        }),
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true,
        }),
    }
}

async fn run(
    daemon: &Daemon,
    id: &BotId,
    generation: u64,
    name: &str,
    arguments: Value,
) -> Result<Reply, String> {
    let tool = Tool::parse(name, arguments)?;
    let bot = about(daemon, id)?;
    let call = daemon.browsers.begin(id).await;
    if let Tool::AskOwner { task } = &tool {
        let session = call.running().ok_or(
            "Your browser is not open. Open the page where you need the owner with browser_open \
             first.",
        )?;
        ask_owner(daemon, id, generation, &session, task).await?;
        // What they did last reaches the browser before the bot reads it.
        daemon.browsers.wait_for_owner(id, OWNER_WAIT).await;
        let session = call.running().ok_or(
            "The owner is done, but your browser closed meanwhile: they may have used it in a \
             window of its own. Open the page again with browser_open; the logins they made \
             stay.",
        )?;
        let done = "The owner is done and gave your browser back. This is the page now.";
        let mut notes = vec![done.to_owned()];
        notes.extend(switched(session.take_switched()).into_iter().flatten());
        return answer(&session, &bot, 0, Some(notes)).await;
    }
    // The owner has the browser: wait until they give it back.
    if !daemon.browsers.wait_for_owner(id, OWNER_WAIT).await {
        return Err(
            "The owner is using your browser and has not given it back. Try again later, or ask \
             them with browser_ask_owner if you need them."
                .to_owned(),
        );
    }
    if let Tool::Close = tool {
        daemon.browsers.close(id);
        return Ok(Reply::Text(
            "Your browser is closed. Its cookies and logins stay for next time.".to_owned(),
        ));
    }
    if let Tool::Open { url } = &tool {
        let (url, site, label) = match sites::resolve(url, &bot.workspace, &bot.folders)? {
            sites::Place::Web { url, site } => {
                let label = site.clone();
                (url, site, label)
            }
            sites::Place::File { url } => {
                let label = url.rsplit('/').next().map(str::to_owned);
                (url, None, label)
            }
        };
        if let Some(site) = &site {
            allowed(daemon, id, generation, &bot, site, &url).await?;
        }
        let downloads = bot.workspace.join("downloads");
        let session = call
            .session(&downloads)
            .await
            .map_err(|err| unavailable(&err))?;
        let moved = session.take_switched();
        let result = session.open(&url).await;
        report(
            daemon,
            id,
            BrowserActionKind::Open,
            &Done {
                label,
                ..Done::default()
            },
        );
        result.map_err(|err| describe(&err))?;
        return answer(&session, &bot, 0, switched(moved)).await;
    }

    let session = call
        .running()
        .ok_or("Your browser is not open. Open a page with browser_open first.")?;
    // On a tab the owner left open, the bot reads before it acts.
    let moved = session.take_switched();
    if moved && tool.acts() {
        return Err(NOT_DONE.to_owned());
    }
    let url = session
        .lock()
        .active()
        .map(|tab| tab.url.clone())
        .unwrap_or_default();
    page_allowed(daemon, id, generation, &bot, &session, &url).await?;
    let (kind, done) = match tool {
        Tool::Look { from } => return answer(&session, &bot, from, switched(moved)).await,
        Tool::Screenshot => {
            let data = session.screenshot().await.map_err(|err| describe(&err))?;
            let reading = session.read(0).await.map_err(|err| describe(&err))?;
            let mut line = format!("Page: {}\nURL: {}", reading.title, reading.url);
            if moved {
                line = format!("{line}\n{SWITCHED}");
            }
            return Ok(Reply::Picture(data, line));
        }
        // The cursor goes first; these report before they act.
        Tool::Click { reference } => {
            let kind = BrowserActionKind::Click;
            let aim = session
                .aim(&reference)
                .await
                .map_err(|err| describe(&err))?;
            point(daemon, id, kind, &aim).await;
            (kind, session.click(aim).await)
        }
        Tool::Type {
            reference,
            text,
            submit,
        } => {
            let kind = BrowserActionKind::Type;
            let aim = session
                .aim_field(&reference)
                .await
                .map_err(|err| describe(&err))?;
            point(daemon, id, kind, &aim).await;
            (kind, session.type_text(aim, &text, submit).await)
        }
        Tool::Select { reference, option } => {
            let kind = BrowserActionKind::Select;
            let aim = session
                .aim(&reference)
                .await
                .map_err(|err| describe(&err))?;
            point(daemon, id, kind, &aim).await;
            (kind, session.choose(aim, &option).await)
        }
        Tool::Press { key } => {
            let label = Some(key.key.trim().to_owned()).filter(|key| !key.is_empty());
            let result = session.press(key).await.map(|()| Done {
                label: label.or(Some("Space".to_owned())),
                ..Done::default()
            });
            (BrowserActionKind::Press, result)
        }
        Tool::Scroll { to, reference } => {
            let result = match (reference, to) {
                (Some(reference), _) => session.scroll_to(&reference).await,
                (None, to) => session.scroll(to.unwrap_or(Scroll::Down)).await,
            };
            (BrowserActionKind::Scroll, result)
        }
        Tool::Back => {
            let went = session.back().await.map_err(|err| describe(&err))?;
            if !went {
                return Err("There is no page to go back to.".to_owned());
            }
            (BrowserActionKind::Back, Ok(Done::default()))
        }
        Tool::Open { .. } | Tool::Close | Tool::AskOwner { .. } => unreachable!("handled above"),
    };
    let done = done.map_err(|err| describe(&err))?;
    let pointed = matches!(
        kind,
        BrowserActionKind::Click | BrowserActionKind::Type | BrowserActionKind::Select
    );
    if !pointed {
        report(daemon, id, kind, &done);
    }
    let mut notes = Vec::new();
    if let Some(cover) = &done.cover {
        notes.push(format!(
            "The click landed on {cover}, which was on top of the element."
        ));
    }
    if let Some(chosen) = &done.chosen {
        notes.push(format!("Chose \"{chosen}\"."));
    }
    answer(&session, &bot, 0, Some(notes)).await
}

/// The note for the bot when the owner left it on another tab.
fn switched(moved: bool) -> Option<Vec<String>> {
    moved.then(|| vec![SWITCHED.to_owned()])
}

fn about(daemon: &Daemon, id: &BotId) -> Result<Bot, String> {
    let store = daemon.store();
    let (crew, record) = bots::active(&store, id).map_err(explain)?;
    let workspace = daemon.paths.bot_workspace(&crew.slug, &record.slug);
    let work = daemon.paths.work_folder(&crew);
    Ok(Bot {
        mode: record.permission_mode,
        folders: vec![workspace.clone(), work],
        workspace,
    })
}
