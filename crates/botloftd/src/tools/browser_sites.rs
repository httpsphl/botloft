//! Which sites a bot may use in its browser (spec 21.5): known ones, those
//! the owner allows when the bot's mode asks first, and never a file
//! outside its folders.

use botloft_core::chat::BROWSER_SITE_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::PermissionMode;
use serde_json::json;

use super::browser::Bot;
use super::calls::explain;
use crate::approvals::{self, Answer};
use crate::browser::{Session, sites};
use crate::state::Daemon;

/// Whether the bot may use `site`, asking the owner the first time when
/// its mode asks first (spec 21.5).
pub(super) async fn allowed(
    daemon: &Daemon,
    id: &BotId,
    generation: u64,
    bot: &Bot,
    site: &str,
    url: &str,
) -> Result<(), String> {
    if matches!(
        bot.mode,
        PermissionMode::Auto | PermissionMode::BypassPermissions
    ) {
        return Ok(());
    }
    let known = daemon
        .store()
        .browser_sites(id)
        .map_err(|err| explain(err.into()))?;
    if known.iter().any(|allowed| sites::covers(allowed, site)) {
        return Ok(());
    }
    let input = json!({ "site": site, "url": url });
    match approvals::ask(daemon, id, generation, BROWSER_SITE_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            daemon
                .store()
                .allow_browser_site(id, site, daemon.clock.now_ms())
                .map_err(|err| explain(err.into()))?;
            Ok(())
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Err(format!(
                "The owner did not allow {site} in your browser.{why} Do without it or ask them."
            ))
        }
        Some(Answer::Expired) => Err(format!(
            "The owner did not answer about {site} in time. Try again later or do without it."
        )),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

/// The page the bot is about to act on: a file outside its folders is
/// closed, and a site it may not use yet is asked about.
pub(super) async fn page_allowed(
    daemon: &Daemon,
    id: &BotId,
    generation: u64,
    bot: &Bot,
    session: &Session,
    url: &str,
) -> Result<(), String> {
    if sites::is_file(url) {
        return outside_file(session, bot, url).await;
    }
    match sites::site_of(url) {
        Some(site) => allowed(daemon, id, generation, bot, &site, url).await,
        None => Ok(()),
    }
}

pub(super) async fn outside_file(session: &Session, bot: &Bot, url: &str) -> Result<(), String> {
    if sites::file_allowed(url, &bot.folders) {
        return Ok(());
    }
    let _ = session.open("about:blank").await;
    Err(
        "That page is a file outside your folders, so it was closed. Open files only from your \
         own folder or the crew's work folder."
            .to_owned(),
    )
}
