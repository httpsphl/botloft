//! The outline in the bot's color around the window it is using (spec
//! 24.9): it shows when a bot reads or acts in a window and follows the
//! window while the bot's turn goes on, until the bot leaves the desktop
//! alone for a minute, its turn ends or the owner stops it. One window at
//! a time: the one used last. The bot's cursor goes with it, at the point
//! of its last action in that window.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, DesktopActionKind, DesktopState};
use tokio::sync::broadcast::error::RecvError;

use super::notice::color;
use crate::platform::desktop::{self, ScreenCursor};
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
    name: String,
    /// Where the bot acted last in the window.
    cursor: Option<ScreenCursor>,
    used: Instant,
    /// When the bot's turn ended, if it did.
    turn_over: Option<Instant>,
}

/// What the outline shows now: the window, the bot's color and cursor.
type Shown = (u64, (u8, u8, u8), Option<ScreenCursor>);

/// What the outline does, apart from the screen.
#[derive(Debug, Default)]
struct Follow {
    outlined: Option<Outlined>,
    /// Each bot's last use, by the state's `at`: a state with the same one
    /// is not a use.
    last: HashMap<BotId, i64>,
}

impl Follow {
    /// A bot read or acted in a window, or the owner stopped it; `who`
    /// gives the bot's color and name.
    fn changed(
        &mut self,
        state: &DesktopState,
        who: impl FnOnce() -> ((u8, u8, u8), String),
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
        let (color, name, before) = match self.outlined.take() {
            Some(outlined) if mine => {
                let before = outlined.cursor.filter(|_| outlined.window == window.id);
                (outlined.color, outlined.name, before)
            }
            _ => {
                let (color, name) = who();
                (color, name, None)
            }
        };
        // Only reading leaves the cursor where it was.
        let cursor = match &state.action {
            Some(action) => action.x.zip(action.y).map(|(x, y)| ScreenCursor {
                x,
                y,
                click: action.kind == DesktopActionKind::Click,
                typing: action.kind == DesktopActionKind::Type,
                name: name.clone(),
                at,
            }),
            None => before,
        };
        self.outlined = Some(Outlined {
            bot: state.bot_id.clone(),
            window: window.id,
            color,
            name,
            cursor,
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
    fn now(&mut self, now: Instant) -> Option<Shown> {
        let outlined = self.outlined.as_ref()?;
        let over = outlined
            .turn_over
            .is_some_and(|ended| now.duration_since(ended) >= AFTER_TURN)
            || now.duration_since(outlined.used) >= IDLE_FOR;
        if over {
            self.outlined = None;
            return None;
        }
        Some((outlined.window, outlined.color, outlined.cursor.clone()))
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
                Some((window, color, cursor)) => {
                    desktop::outline_show(window, color, cursor);
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
                    let who = || {
                        let bot = daemon.store().bot(&state.bot_id).ok().flatten();
                        let (hex, name) = bot.map(|bot| (bot.color, bot.name)).unwrap_or_default();
                        (color(&hex), name)
                    };
                    follow.changed(&state, who, Instant::now());
                    // The cursor sets off now, not at the next tick.
                    if let Some((window, color, cursor)) = follow.now(Instant::now()) {
                        desktop::outline_show(window, color, cursor);
                        shown = true;
                    }
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
    use botloft_core::protocol::{DesktopAction, DesktopWindow};

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

    /// The window and color outlined at `now`, without the cursor.
    fn spot(follow: &mut Follow, now: Instant) -> Option<(u64, (u8, u8, u8))> {
        follow.now(now).map(|(window, color, _)| (window, color))
    }

    #[test]
    fn the_cursor_is_at_the_last_action_and_stays_while_the_bot_reads() {
        let mut follow = Follow::default();
        let (scout, start) = (BotId::generate(), Instant::now());
        let mut clicked = used(&scout, 7, 1);
        clicked.action = Some(DesktopAction {
            kind: DesktopActionKind::Click,
            target: "Save".to_owned(),
            option: None,
            x: Some(0.25),
            y: Some(0.5),
        });
        follow.changed(&clicked, || (BLUE, "Scout".to_owned()), start);
        let cursor = follow
            .now(start)
            .and_then(|(_, _, cursor)| cursor)
            .expect("cursor");
        assert_eq!(
            (cursor.x, cursor.y, cursor.click, cursor.typing),
            (0.25, 0.5, true, false)
        );
        assert_eq!((cursor.name.as_str(), cursor.at), ("Scout", 1));

        follow.changed(&used(&scout, 7, 2), || (BLUE, "Scout".to_owned()), start);
        let kept = follow.now(start).and_then(|(_, _, cursor)| cursor);
        assert_eq!(kept.map(|cursor| cursor.at), Some(1), "reading leaves it");
        // Another window starts without one.
        follow.changed(&used(&scout, 8, 3), || (BLUE, "Scout".to_owned()), start);
        assert_eq!(follow.now(start).and_then(|(_, _, cursor)| cursor), None);
    }

    #[test]
    fn the_outline_follows_the_window_in_use_until_the_turn_ends() {
        let mut follow = Follow::default();
        let (scout, start) = (BotId::generate(), Instant::now());
        assert_eq!(spot(&mut follow, start), None);
        follow.changed(&used(&scout, 7, 1), || (BLUE, "Scout".to_owned()), start);
        assert_eq!(spot(&mut follow, start), Some((7, BLUE)));

        // Its turn ends: a moment more, then gone.
        follow.turn(&scout, false, start);
        assert_eq!(
            spot(&mut follow, start + Duration::from_secs(1)),
            Some((7, BLUE))
        );
        assert_eq!(spot(&mut follow, start + AFTER_TURN), None);
    }

    #[test]
    fn a_minute_away_from_the_desktop_or_a_stop_takes_it_away() {
        let mut follow = Follow::default();
        let (scout, start) = (BotId::generate(), Instant::now());
        follow.changed(&used(&scout, 7, 1), || (BLUE, "Scout".to_owned()), start);
        assert_eq!(spot(&mut follow, start + IDLE_FOR), None);

        follow.changed(&used(&scout, 7, 2), || (BLUE, "Scout".to_owned()), start);
        let mut stopped = used(&scout, 7, 2);
        stopped.stopped = true;
        follow.changed(&stopped, || (BLUE, "Scout".to_owned()), start);
        assert_eq!(spot(&mut follow, start), None);
        // Letting it go on is not a use.
        stopped.stopped = false;
        follow.changed(&stopped, || (BLUE, "Scout".to_owned()), start);
        assert_eq!(spot(&mut follow, start), None);
    }

    #[test]
    fn the_window_used_last_is_the_one_outlined() {
        let mut follow = Follow::default();
        let (scout, writer, start) = (BotId::generate(), BotId::generate(), Instant::now());
        follow.changed(&used(&scout, 7, 1), || (BLUE, "Scout".to_owned()), start);
        follow.changed(
            &used(&writer, 9, 2),
            || ((1, 2, 3), "Writer".to_owned()),
            start,
        );
        assert_eq!(spot(&mut follow, start), Some((9, (1, 2, 3))));
        // Another bot's turn ending leaves it.
        follow.turn(&scout, false, start);
        assert_eq!(spot(&mut follow, start + AFTER_TURN), Some((9, (1, 2, 3))));
    }
}
