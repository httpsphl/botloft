//! The browser tools (spec 21.4): each acts in the bot's own browser, asks
//! the owner before a new site when the bot's mode asks first (spec 21.5),
//! waits while the owner has it (spec 21.10), and answers with the page as
//! it is after the action.

use std::path::PathBuf;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserActionKind, PermissionMode};
use serde_json::{Value, json};
use tracing::debug;

use super::browser_args::Tool;
use super::browser_help::ask_owner;
use super::browser_reply::{answer, describe, report, unavailable};
use super::browser_sites::{allowed, page_allowed};
use super::calls::explain;
use crate::browser::{Done, OWNER_WAIT, Scroll, sites};
use crate::service::bots;
use crate::state::Daemon;

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
        let session = call.running().ok_or(
            "The owner is done, but your browser closed meanwhile. Open the page again with \
             browser_open.",
        )?;
        let done = "The owner is done and gave your browser back. This is the page now.";
        return answer(&session, &bot, 0, Some(vec![done.to_owned()])).await;
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
        return answer(&session, &bot, 0, None).await;
    }

    let session = call
        .running()
        .ok_or("Your browser is not open. Open a page with browser_open first.")?;
    let url = session
        .lock()
        .active()
        .map(|tab| tab.url.clone())
        .unwrap_or_default();
    page_allowed(daemon, id, generation, &bot, &session, &url).await?;
    let (kind, done) = match tool {
        Tool::Look { from } => return answer(&session, &bot, from, None).await,
        Tool::Screenshot => {
            let data = session.screenshot().await.map_err(|err| describe(&err))?;
            let reading = session.read(0).await.map_err(|err| describe(&err))?;
            let line = format!("Page: {}\nURL: {}", reading.title, reading.url);
            return Ok(Reply::Picture(data, line));
        }
        Tool::Click { reference } => (BrowserActionKind::Click, session.click(&reference).await),
        Tool::Type {
            reference,
            text,
            submit,
        } => (
            BrowserActionKind::Type,
            session.type_text(&reference, &text, submit).await,
        ),
        Tool::Select { reference, option } => (
            BrowserActionKind::Select,
            session.choose(&reference, &option).await,
        ),
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
    report(daemon, id, kind, &done);
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
