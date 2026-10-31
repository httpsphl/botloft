//! What the owner allowed only for a bot's turn (spec 24.2, 21.5): an app
//! on the desktop or a site in the browser, answered "Only this time".
//! Kept in memory, never saved, and forgotten when the turn ends.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use botloft_core::ids::{BotId, DesktopGrantId};
use botloft_core::protocol::{BotState, DesktopGrant, DesktopLevel, DesktopScope};
use tokio::sync::broadcast::error::RecvError;

use crate::browser::sites;
use crate::state::{Daemon, Event};

#[derive(Default)]
pub struct ThisTurn {
    allowed: Mutex<HashMap<BotId, Allowed>>,
}

#[derive(Default)]
struct Allowed {
    /// As grants, for the desktop tools to read with the saved ones: never
    /// with the real mouse and keyboard, nor with the owner away.
    apps: Vec<DesktopGrant>,
    sites: Vec<String>,
}

impl ThisTurn {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Allowed>> {
        self.allowed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Lets `bot` see, or see and use, the app at `path` until its turn
    /// ends. Like a saved grant, allowing it again only raises the level.
    pub fn allow_app(&self, bot: &BotId, path: &str, name: &str, level: DesktopLevel, now: i64) {
        let mut allowed = self.lock();
        let apps = &mut allowed.entry(bot.clone()).or_default().apps;
        let same = apps.iter_mut().find(|grant| {
            grant
                .app_path
                .as_deref()
                .is_some_and(|known| known.eq_ignore_ascii_case(path))
        });
        if let Some(same) = same {
            if level == DesktopLevel::Act {
                same.level = DesktopLevel::Act;
            }
            return;
        }
        apps.push(DesktopGrant {
            id: DesktopGrantId::generate(),
            bot_id: bot.clone(),
            scope: DesktopScope::App,
            app_path: Some(path.to_owned()),
            app_name: Some(name.to_owned()),
            level,
            real_input: false,
            unattended: false,
            accepted_risks_at: None,
            created_at: now,
        });
    }

    /// The apps `bot` may use until its turn ends.
    pub fn apps(&self, bot: &BotId) -> Vec<DesktopGrant> {
        self.lock()
            .get(bot)
            .map(|allowed| allowed.apps.clone())
            .unwrap_or_default()
    }

    /// Lets `bot` use `site` (and the sites under it) until its turn ends.
    pub fn allow_site(&self, bot: &BotId, site: &str) {
        let mut allowed = self.lock();
        let sites = &mut allowed.entry(bot.clone()).or_default().sites;
        if !sites.iter().any(|known| known == site) {
            sites.push(site.to_owned());
        }
    }

    pub fn site(&self, bot: &BotId, site: &str) -> bool {
        self.lock()
            .get(bot)
            .is_some_and(|allowed| allowed.sites.iter().any(|known| sites::covers(known, site)))
    }

    /// The bot's turn ended: what was allowed for it goes.
    pub fn end(&self, bot: &BotId) {
        self.lock().remove(bot);
    }

    fn end_all(&self) {
        self.lock().clear();
    }
}

/// Forgets what each bot was allowed for its turn as soon as the turn
/// ends, that is when its state leaves busy and waiting for the owner
/// (spec 7.1).
pub async fn forget_turns(daemon: Arc<Daemon>) {
    let mut events = daemon.subscribe();
    loop {
        match events.recv().await {
            Ok(Event::BotState(change))
                if !matches!(change.state, BotState::Busy | BotState::NeedsApproval) =>
            {
                daemon.approvals.this_turn.end(&change.bot_id);
            }
            Ok(_) => {}
            // A turn may have ended unseen: asking again is the safe side.
            Err(RecvError::Lagged(_)) => daemon.approvals.this_turn.end_all(),
            Err(RecvError::Closed) => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_app_or_a_site_holds_for_the_turn_and_goes_with_it() {
        let turn = ThisTurn::default();
        let (scout, writer) = (BotId::generate(), BotId::generate());
        let excel = r"C:\Office\EXCEL.EXE";
        turn.allow_app(&scout, excel, "Excel", DesktopLevel::See, 10);
        // Again, even in other letters, only raises the level.
        turn.allow_app(
            &scout,
            &excel.to_lowercase(),
            "Excel",
            DesktopLevel::Act,
            20,
        );
        turn.allow_app(&scout, excel, "Excel", DesktopLevel::See, 30);
        let apps = turn.apps(&scout);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].level, DesktopLevel::Act);
        assert!(!apps[0].real_input && !apps[0].unattended);
        assert!(turn.apps(&writer).is_empty());

        turn.allow_site(&scout, "wikipedia.org");
        assert!(turn.site(&scout, "pt.wikipedia.org"));
        assert!(!turn.site(&scout, "example.com"));
        assert!(!turn.site(&writer, "wikipedia.org"));

        turn.end(&scout);
        assert!(turn.apps(&scout).is_empty());
        assert!(!turn.site(&scout, "wikipedia.org"));
    }
}
