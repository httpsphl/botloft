//! A bot asking the owner for a hand in its browser (spec 21.10): the
//! request in the chat, and the owner taking over until they are done.

use botloft_core::chat::BROWSER_HELP_TOOL;
use botloft_core::ids::BotId;
use serde_json::json;

use crate::approvals::{self, Answer};
use crate::browser::{Session, sites};
use crate::state::Daemon;

/// Asks the owner to do `task` on the page and waits. `Ok` once they say
/// they are done.
pub(super) async fn ask_owner(
    daemon: &Daemon,
    id: &BotId,
    generation: u64,
    session: &Session,
    task: &str,
) -> Result<(), String> {
    let url = session
        .lock()
        .active()
        .map(|tab| tab.url.clone())
        .unwrap_or_default();
    let input = json!({ "task": task, "url": url, "site": sites::site_of(&url) });
    let asking = daemon.browsers.asking(id, task);
    let answer = approvals::ask_with(
        daemon,
        id,
        generation,
        BROWSER_HELP_TOOL,
        &input,
        "",
        |approval| {
            asking.opened(approval);
        },
    )
    .await;
    drop(asking);
    match answer {
        Some(Answer::Allowed { .. }) => Ok(()),
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Err(format!(
                "The owner did not do it.{why} Find another way, or ask them in the chat what \
                 they prefer."
            ))
        }
        Some(Answer::Expired) => Err(
            "The owner did not come in time. Tell them in the chat what you need and ask again \
             later."
                .to_owned(),
        ),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}
