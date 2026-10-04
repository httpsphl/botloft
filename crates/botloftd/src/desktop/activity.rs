//! What each bot does on the owner's desktop, for its panel (spec 24.9):
//! the window it is using, what it did last, and whether the owner stopped
//! it. A stopped bot's desktop tools refuse until the owner lets it go on.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopState, DesktopWindow};

use crate::platform::desktop::Window;

/// What the bot reads when the owner stopped it.
pub const STOPPED: &str = "The owner stopped you from using their desktop. Do not try again: \
    tell them in the chat what you were doing, and they let you go on when they want.";

#[derive(Default)]
pub struct Activity {
    by_bot: Mutex<HashMap<BotId, DesktopState>>,
}

fn blank(bot: &BotId) -> DesktopState {
    DesktopState {
        bot_id: bot.clone(),
        window: None,
        action: None,
        at: None,
        stopped: false,
    }
}

impl Activity {
    fn change(&self, bot: &BotId, how: impl FnOnce(&mut DesktopState)) -> DesktopState {
        let mut all = self.by_bot.lock().unwrap_or_else(PoisonError::into_inner);
        let state = all.entry(bot.clone()).or_insert_with(|| blank(bot));
        how(state);
        state.clone()
    }

    pub fn state(&self, bot: &BotId) -> DesktopState {
        self.by_bot
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(bot)
            .cloned()
            .unwrap_or_else(|| blank(bot))
    }

    /// The window the bot is using now, by its number.
    pub fn window(&self, bot: &BotId) -> Option<u64> {
        self.state(bot).window.map(|window| window.id)
    }

    /// The bot read or acted in `window` at `now`; `action` says what it
    /// did, when it did something.
    pub fn used(
        &self,
        bot: &BotId,
        window: &Window,
        action: Option<String>,
        now: i64,
    ) -> DesktopState {
        self.change(bot, |state| {
            state.window = Some(DesktopWindow {
                id: window.id,
                title: window.title.clone(),
                app: window.app.name.clone(),
            });
            state.action = action;
            state.at = Some(now);
        })
    }

    pub fn stopped(&self, bot: &BotId) -> bool {
        self.state(bot).stopped
    }

    /// Stops the bot, or lets it go on.
    pub fn set_stopped(&self, bot: &BotId, stopped: bool) -> DesktopState {
        self.change(bot, |state| state.stopped = stopped)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::platform::desktop::App;

    #[test]
    fn a_bot_shows_its_window_and_last_action_and_stays_stopped_until_it_may_go_on() {
        let activity = Activity::default();
        let bot = BotId::generate();
        assert_eq!(activity.state(&bot), blank(&bot));
        let window = Window {
            id: 7,
            title: "Visits".to_owned(),
            class: "Any".to_owned(),
            app: App {
                path: PathBuf::from(r"C:\Apps\clinic.exe"),
                name: "Clinic".to_owned(),
            },
            minimized: false,
            elevated: false,
        };
        let used = activity.used(
            &bot,
            &window,
            Some("Clicked button \"Save\".".to_owned()),
            9,
        );
        assert_eq!(activity.window(&bot), Some(7));
        assert_eq!(
            used.window.map(|window| window.app),
            Some("Clinic".to_owned())
        );
        assert_eq!(used.at, Some(9));

        assert!(!activity.stopped(&bot));
        assert!(activity.set_stopped(&bot, true).stopped);
        assert!(activity.stopped(&bot));
        // Reading again does not let it go on.
        activity.used(&bot, &window, None, 10);
        assert!(activity.stopped(&bot));
        assert!(!activity.set_stopped(&bot, false).stopped);
    }
}
