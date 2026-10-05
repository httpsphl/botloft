//! The owner opens a bot's closed browser for themselves (spec 21.10): on
//! the page it showed last, without the bot, which is neither woken nor
//! told. The app then takes it into the owner's hands.

use std::path::Path;

use botloft_core::ids::BotId;
use tracing::debug;

use super::{BrowserError, Browsers, lock};

impl Browsers {
    /// Starts the bot's browser if it is closed and opens the last page it
    /// showed again; a running one stays as it is.
    pub async fn open_for_owner(&self, bot: &BotId, downloads: &Path) -> Result<(), BrowserError> {
        let call = self.begin(bot).await;
        if call.running().is_some() {
            return Ok(());
        }
        let last = lock(&self.slots)
            .get(bot)
            .and_then(|slot| slot.last_url.clone());
        let session = call.session(downloads).await?;
        if let Some(url) = last
            && let Err(err) = session.open(&url).await
        {
            debug!(bot = %bot, "browser: the last page did not open again: {err}");
        }
        debug!(bot = %bot, "browser: opened for the owner");
        Ok(())
    }
}

/// Keeps `url` as the last page of the bot's browser, unless it is blank.
pub(super) fn remember(slots: &super::Slots, bot: &BotId, url: Option<&str>) {
    let Some(url) = url.filter(|url| !url.is_empty() && *url != "about:blank") else {
        return;
    };
    if let Some(slot) = lock(slots).get_mut(bot) {
        slot.last_url = Some(url.to_owned());
    }
}
