//! An app connection watching a bot's browser (spec 21.7). While anyone
//! watches, the active tab sends frames; when the last one stops, it stops.

use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::BrowserFrame;
use tokio::sync::watch;

use super::{Browsers, Slots, lock};

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
        sync(Arc::clone(&self.slots), bot.clone());
        Watching {
            slots: Arc::clone(&self.slots),
            bot: bot.clone(),
            frames,
        }
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        if let Some(slot) = lock(&self.slots).get_mut(&self.bot) {
            slot.watchers = slot.watchers.saturating_sub(1);
        }
        sync(Arc::clone(&self.slots), self.bot.clone());
    }
}

/// Turns the frames on or off to match whether anyone watches. It reads
/// the count when it runs, so calls that finish out of order still end on
/// the right answer.
fn sync(slots: Slots, bot: BotId) {
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return;
    };
    runtime.spawn(async move {
        let found = lock(&slots).get(&bot).and_then(|slot| {
            let session = slot.session.as_ref()?.1.clone();
            Some((session, slot.watchers > 0))
        });
        if let Some((session, on)) = found {
            session.set_watching(on).await;
        }
    });
}
