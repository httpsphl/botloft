//! An app connection watching a bot's browser (spec 21.7). While anyone
//! watches, the active tab sends frames and the page is the size their
//! panel asks for (spec 21.3); when the last one stops, both stop.

use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::BrowserFrame;
use tokio::sync::watch;

use super::{Browsers, Slots, Viewport, lock};

/// Held by the connection; dropping it stops watching.
pub struct Watching {
    slots: Slots,
    pub bot: BotId,
    /// The newest frame; `changed()` wakes on each new one.
    pub frames: watch::Receiver<Option<Arc<BrowserFrame>>>,
}

impl Browsers {
    pub fn watch(&self, bot: &BotId) -> Watching {
        let frames = {
            let mut slots = lock(&self.slots);
            let slot = self.slot(&mut slots, bot);
            slot.watchers += 1;
            slot.frames.subscribe()
        };
        sync_soon(&self.slots, bot);
        Watching {
            slots: Arc::clone(&self.slots),
            bot: bot.clone(),
            frames,
        }
    }

    /// Someone has the bot's browser open in the app.
    pub fn watched(&self, bot: &BotId) -> bool {
        lock(&self.slots)
            .get(bot)
            .is_some_and(|slot| slot.watchers > 0)
    }

    /// The room a watching app has for the page, `width` by `height`: the
    /// page takes its shape, now or when the browser opens (spec 21.3).
    pub fn resize(&self, bot: &BotId, width: u32, height: u32) {
        {
            let mut slots = lock(&self.slots);
            self.slot(&mut slots, bot).viewport = Viewport::fitting(width, height);
        }
        sync_soon(&self.slots, bot);
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        if let Some(slot) = lock(&self.slots).get_mut(&self.bot) {
            slot.watchers = slot.watchers.saturating_sub(1);
            if slot.watchers == 0 {
                // No panel to fit anymore.
                slot.viewport = Viewport::default();
            }
        }
        sync_soon(&self.slots, &self.bot);
    }
}

/// Makes the running browser match what the apps want: frames while anyone
/// watches, and the page the size they asked for. One at a time, each
/// reading what is wanted when its turn comes, so changes that finish out
/// of order still end on the right answer.
pub(super) async fn sync(slots: Slots, bot: BotId) {
    let running = lock(&slots).get(&bot).and_then(|slot| {
        slot.session
            .as_ref()
            .map(|(_, session)| Arc::clone(session))
    });
    let Some(session) = running else {
        return;
    };
    let _turn = session.syncing.lock().await;
    let wanted = lock(&slots)
        .get(&bot)
        .map(|slot| (slot.watchers > 0, slot.viewport));
    if let Some((on, viewport)) = wanted {
        session.set_watching(on).await;
        session.set_viewport(viewport).await;
    }
}

fn sync_soon(slots: &Slots, bot: &BotId) {
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(sync(Arc::clone(slots), bot.clone()));
    }
}
