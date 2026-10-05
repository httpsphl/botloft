//! What bots did on the owner's desktop while the owner was away (spec
//! 24.8): one line per bot and app, kept until the owner dismisses them,
//! so the app tells them when they are back. In memory: what the bot did
//! is in its chat anyway.

use std::sync::{Mutex, PoisonError};

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::DesktopAwayUse;

#[derive(Default)]
pub struct AwayUses {
    uses: Mutex<Vec<DesktopAwayUse>>,
}

impl AwayUses {
    /// `bot` used `app` at `now`; `item` is the newest of its chat then.
    /// The list now.
    pub fn record(
        &self,
        bot: &BotId,
        app: &str,
        now: i64,
        item: Option<ChatItemId>,
    ) -> Vec<DesktopAwayUse> {
        let mut uses = self.uses.lock().unwrap_or_else(PoisonError::into_inner);
        match uses
            .iter_mut()
            .find(|used| used.bot_id == *bot && used.app == app)
        {
            Some(used) => used.until = now,
            None => uses.push(DesktopAwayUse {
                bot_id: bot.clone(),
                app: app.to_owned(),
                from: now,
                until: now,
                item_id: item,
            }),
        }
        uses.clone()
    }

    /// Oldest first.
    pub fn list(&self) -> Vec<DesktopAwayUse> {
        self.uses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The owner saw them.
    pub fn clear(&self) {
        self.uses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_line_per_bot_and_app_until_the_owner_dismisses_them() {
        let away = AwayUses::default();
        let (scout, writer) = (BotId::generate(), BotId::generate());
        let item = ChatItemId::generate();
        away.record(&scout, "Excel", 10, Some(item.clone()));
        away.record(&scout, "Excel", 20, None);
        away.record(&writer, "Excel", 30, None);
        let uses = away.record(&scout, "Notepad", 40, None);
        assert_eq!(uses.len(), 3);
        assert_eq!((uses[0].from, uses[0].until), (10, 20));
        assert_eq!(uses[0].item_id, Some(item), "the chat where it began");
        assert_eq!(uses[1].bot_id, writer);
        away.clear();
        assert!(away.list().is_empty());
    }
}
