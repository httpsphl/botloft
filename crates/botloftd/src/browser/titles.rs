//! Each tab's title, for the owner's list of tabs (spec 21.7). Edge without
//! a window tells a tab's new address but never its title (spec 19), so the
//! daemon reads `document.title` when the page loads.

use serde_json::json;
use tracing::debug;

use super::read::WORLD;
use super::session::Session;

/// Longest title kept, in characters.
const TITLE_MAX: usize = 200;

/// Whether a page's event may have come with a new title.
pub(super) fn retitles(method: &str) -> bool {
    matches!(
        method,
        "Page.domContentEventFired" | "Page.loadEventFired" | "Page.navigatedWithinDocument"
    )
}

impl Session {
    /// Reads the title of the tab whose page session is `page`, in the
    /// isolated world, where the page's scripts cannot answer for it.
    pub(super) async fn retitle(&self, page: &str) {
        let target = self.lock().by_session(page).map(|tab| tab.target.clone());
        let Some(target) = target else {
            return;
        };
        let world = self
            .cdp
            .call(
                Some(page),
                "Page.createIsolatedWorld",
                json!({ "frameId": target, "worldName": WORLD }),
            )
            .await;
        let read = match world {
            Ok(world) => {
                let params = json!({
                    "expression": "document.title",
                    "contextId": world["executionContextId"],
                    "returnByValue": true,
                });
                self.cdp.call(Some(page), "Runtime.evaluate", params).await
            }
            Err(err) => Err(err),
        };
        let title = match &read {
            Ok(read) => read["result"]["value"].as_str().unwrap_or_default(),
            // Between two documents; the next load reads it.
            Err(err) => {
                debug!("browser: reading a tab's title: {err}");
                return;
            }
        };
        let title = botloft_core::chat::one_line(title, TITLE_MAX);
        let changed = self.lock().by_session(page).is_some_and(|tab| {
            let changed = tab.title != title;
            tab.title = title;
            changed
        });
        if changed {
            self.moved.notify_one();
        }
    }
}
