//! What an agent's rules say about the logins the owner gave it (spec 35.3).

use std::io;
use std::path::Path;

use botloft_core::ids::BotId;
use tracing::warn;

use crate::service::bots;
use crate::state::Daemon;

/// `.claude/rules/logins.md`: the sites its browser is signed in to with
/// the owner's account. Nothing when it has none.
fn write(workspace: &Path, sites: &[String]) -> io::Result<()> {
    let path = workspace.join(".claude").join("rules").join("logins.md");
    if sites.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    let mut text = String::from(
        "# Sites you are signed in to\n\nThe owner signed in to these sites and gave you the \
         login: open them in your browser and you are already in their account, with no \
         sign-in.\n\n",
    );
    for site in sites {
        text.push_str(&format!("- {site}\n"));
    }
    text.push_str(
        "\nYou act there as the owner. Read and search freely; before you post, send, buy, \
         delete or change a setting of the account, ask the owner, unless they already asked \
         you to. If a site asks you to sign in again, tell the owner to sign in again in \
         Botloft under Settings > Logins. For a site you have no login to, ask the owner to \
         sign in and give it to you there; never ask for a password.\n",
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, text)
}

/// Writes the agent's logins rules in `workspace`, at its start.
pub fn write_rules(daemon: &Daemon, bot: &BotId, workspace: &Path) -> io::Result<()> {
    let sites = daemon
        .store()
        .bot_site_logins(bot)
        .map_err(io::Error::other)?;
    write(workspace, &sites)
}

/// The agent's logins changed: its rules say so, and it starts again with
/// them once nothing is in progress, as for its tools (spec 34.4).
pub(super) fn refresh(daemon: &Daemon, bot: &BotId) {
    let workspace = {
        let store = daemon.store();
        match bots::active(&store, bot) {
            Ok((crew, record)) => daemon.paths.bot_workspace(&crew.slug, &record.slug),
            Err(_) => return,
        }
    };
    if let Err(err) = write_rules(daemon, bot, &workspace) {
        warn!(bot = %bot, "logins: could not write the rules: {err}");
    }
    daemon.supervisor.launch_settings_changed(bot);
}
