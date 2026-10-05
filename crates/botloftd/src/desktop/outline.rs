//! The outline in the bot's color around the window it is using (spec
//! 24.9): it shows when a bot reads or acts in a window and follows the
//! window while the bot's turn goes on, until the bot leaves the desktop
//! alone for a minute, its turn ends or the owner stops it. One window at
//! a time: the one used last.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, DesktopState};
use tokio::sync::broadcast::error::RecvError;

use super::notice::color;
use crate::platform::desktop;
use crate::state::{Daemon, Event};

/// How often the outline catches up with its window.
const FOLLOW_EVERY: Duration = Duration::from_millis(250);
/// How long it stays after the bot's turn ended.
const AFTER_TURN: Duration = Duration::from_secs(3);
/// How long it stays with the bot working but away from the desktop.
const IDLE_FOR: Duration = Duration::from_secs(60);

/// The window outlined now.
#[derive(Debug, Clone, PartialEq)]
struct Outlined {
    bot: BotId,
    window: u64,
    color: (u8, u8, u8),
    used: Instant,
    /// When the bot's turn ended, if it did.
    turn_over: Option<Instant>,
}

/// What the outline does, apart from the screen.
#[derive(Debug, Default)]
struct Follow {
    outlined: Option<Outlined>,
    /// Each bot's last use, by the state's `at`: a state with the same one
    /// is not a use.
    last: HashMap<BotId, i64>,
}

impl Follow {
    /// A bot read or acted in a window, or the owner stopped it.
    fn changed(
        &mut self,
        state: &DesktopState,
        color: impl FnOnce() -> (u8, u8, u8),
        now: Instant,
    ) {
        let mine = self
            .outlined
            .as_ref()
            .is_some_and(|outlined| outlined.bot == state.bot_id);
        if state.stopped {
            if mine {
                self.outlined = None;
            }
            return;
        }
        let (Some(window), Some(at)) = (&state.window, state.at) else {
            return;
        };
        // Letting it go on is not a use.
        if self.last.insert(state.bot_id.clone(), at) == Some(at) {
            return;
        }
        let color = match &self.outlined {
            Some(outlined) if mine => outlined.color,
            _ => color(),
        };
        self.outlined = Some(Outlined {
            bot: state.bot_id.clone(),
            window: window.id,
            color,
            used: now,
            turn_over: None,
        });
    }

    /// The bot's turn began or ended.
    fn turn(&mut self, bot: &BotId, working: bool, now: Instant) {
        if let Some(outlined) = self
            .outlined
            .as_mut()
            .filter(|outlined| outlined.bot == *bot)
        {
            outlined.turn_over = (!working).then_some(now);
        }
    }

    /// The window to outline now, if any.
    fn now(&mut self, now: Instant) -> Option<(u64, (u8, u8, u8))> {
        let outlined = self.outlined.as_ref()?;
        let over = outlined
            .turn_over
            .is_some_and(|ended| now.duration_since(ended) >= AFTER_TURN)
            || now.duration_since(outlined.used) >= IDLE_FOR;
        if over {
            self.outlined = None;
            return None;
        }
        Some((outlined.window, outlined.color))
    }
}

fn working(state: BotState) -> bool {
    matches!(state, BotState::Busy | BotState::NeedsApproval)
}

/// Follows what the bots do on the desktop and keeps the outline there.
pub async fn run(daemon: Arc<Daemon>) {
    let mut tick = tokio::time::interval(FOLLOW_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut events = daemon.subscribe();
    let mut follow = Follow::default();
    let mut shown = false;
    loop {
        tokio::select! {
            _ = tick.tick() => match follow.now(Instant::now()) {
                Some((window, color)) => {
                    desktop::outline_show(window, color);
                    shown = true;
                }
                None if shown => {
                    desktop::outline_hide();
                    shown = false;
                }
                None => {}
            },
            event = events.recv() => match event {
                Ok(Event::DesktopChanged(state)) => {
                    let hex = || {
                        let bot = daemon.store().bot(&state.bot_id).ok().flatten();
                        color(&bot.map(|bot| bot.color).unwrap_or_default())
                    };
                    follow.changed(&state, hex, Instant::now());
                }
                Ok(Event::BotState(change)) => {
                    follow.turn(&change.bot_id, working(change.state), Instant::now());
                }
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => return,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::DesktopWindow;

    use super::*;

    fn used(bot: &BotId, window: u64, at: i64) -> DesktopState {
        DesktopState {
            bot_id: bot.clone(),
            window: Some(DesktopWindow {
                id: window,
                title: "Notes".to_owned(),
                app: "Notepad".to_owned(),
            }),
            action: None,
            at: Some(at),
            stopped: false,
        }
    }

    const BLUE: (u8, u8, u8) = (0x5E, 0xC8, 0xFF);

    #[test]
    fn the_outline_follows_the_window_in_use_until_the_turn_ends() {
        let mut follow = Follow::default();
        let (scout, start) = (BotId::generate(), Instant::now());
        assert_eq!(follow.now(start), None);
        follow.changed(&used(&scout, 7, 1), || BLUE, start);
        assert_eq!(follow.now(start), Some((7, BLUE)));

        // Its turn ends: a moment more, then gone.
        follow.turn(&scout, false, start);
        assert_eq!(follow.now(start + Duration::from_secs(1)), Some((7, BLUE)));
        assert_eq!(follow.now(start + AFTER_TURN), None);
    }

    #[test]
    fn a_minute_away_from_the_desktop_or_a_stop_takes_it_away() {
        let mut follow = Follow::default();
        let (scout, start) = (BotId::generate(), Instant::now());
        follow.changed(&used(&scout, 7, 1), || BLUE, start);
        assert_eq!(follow.now(start + IDLE_FOR), None);

        follow.changed(&used(&scout, 7, 2), || BLUE, start);
        let mut stopped = used(&scout, 7, 2);
        stopped.stopped = true;
        follow.changed(&stopped, || BLUE, start);
        assert_eq!(follow.now(start), None);
        // Letting it go on is not a use.
        stopped.stopped = false;
        follow.changed(&stopped, || BLUE, start);
        assert_eq!(follow.now(start), None);
    }

    #[test]
    fn the_window_used_last_is_the_one_outlined() {
        let mut follow = Follow::default();
        let (scout, writer, start) = (BotId::generate(), BotId::generate(), Instant::now());
        follow.changed(&used(&scout, 7, 1), || BLUE, start);
        follow.changed(&used(&writer, 9, 2), || (1, 2, 3), start);
        assert_eq!(follow.now(start), Some((9, (1, 2, 3))));
        // Another bot's turn ending leaves it.
        follow.turn(&scout, false, start);
        assert_eq!(follow.now(start + AFTER_TURN), Some((9, (1, 2, 3))));
    }
}
