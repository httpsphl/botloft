//! Per-bot supervision state and the transition rules of spec 7.2.

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::BotState;
use tokio::time::Instant;

use crate::runtime::{ProcessControl, TermSize};
use crate::terminal::Terminal;

pub(super) struct Slot {
    pub state: BotState,
    /// Generation of the current or last process.
    pub generation: Option<u64>,
    pub running: Option<Running>,
    pub backoff: Backoff,
    pub restart_at: Option<Instant>,
    /// Start the next process without `--continue`.
    pub fresh_next: bool,
    /// Why the running process is being killed, if the daemon asked.
    pub stop: Option<StopIntent>,
    pub size: TermSize,
    pub terminal: Arc<Terminal>,
    /// Where to deliver messages to this generation (spec 9.2); memory only.
    pub inbox: Option<Inbox>,
}

impl Slot {
    pub fn new(ring_buffer_bytes: usize, backoff: Backoff) -> Self {
        Self {
            state: BotState::Offline,
            generation: None,
            running: None,
            backoff,
            restart_at: None,
            fresh_next: false,
            stop: None,
            size: TermSize::default(),
            terminal: Arc::new(Terminal::new(ring_buffer_bytes)),
            inbox: None,
        }
    }
}

pub(super) struct Running {
    pub control: Box<dyn ProcessControl>,
    pub started: Instant,
    /// Started with `--continue`.
    pub resumed: bool,
    pub token_hash: String,
    pub workspace: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StopIntent {
    Restart {
        fresh: bool,
    },
    /// Paused or archived: settle in this state.
    Halt(BotState),
}

/// The bot's inbox, reported by the `SessionStart` hook.
#[derive(Clone, PartialEq, Eq)]
pub struct Inbox {
    pub socket: String,
    pub token: String,
}

impl fmt::Debug for Inbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inbox")
            .field("socket", &self.socket)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// What a hook reported. Payload fields as documented by Claude Code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hook {
    SessionStart {
        inbox: Option<Inbox>,
    },
    PromptSubmit,
    Stop,
    /// `error`: `rate_limit`, `authentication_failed`, ...
    StopFailure {
        error: String,
    },
    /// `notification_type`: `permission_prompt`, `idle_prompt`, ...
    Notification {
        kind: String,
    },
    SessionEnd,
}

/// The state a hook moves the bot to, if it changes it (spec 7.2).
pub(super) fn state_after_hook(current: BotState, hook: &Hook) -> Option<BotState> {
    let next = match hook {
        Hook::SessionStart { .. } | Hook::Stop => BotState::Idle,
        Hook::PromptSubmit => BotState::Busy,
        Hook::StopFailure { error } => match error.as_str() {
            "rate_limit" => BotState::RateLimited,
            "authentication_failed" | "oauth_org_not_allowed" => BotState::AuthError,
            // The turn ended; the session is still usable.
            _ => BotState::Idle,
        },
        Hook::Notification { kind } if kind == "permission_prompt" => BotState::NeedsApproval,
        Hook::Notification { .. } | Hook::SessionEnd => return None,
    };
    (next != current).then_some(next)
}

/// Exponential restart delay with jitter (spec 7.3).
#[derive(Debug, Clone)]
pub(super) struct Backoff {
    initial: Duration,
    max: Duration,
    attempts: u32,
}

impl Backoff {
    pub fn new(initial: Duration, max: Duration) -> Self {
        Self {
            initial,
            max: max.max(initial),
            attempts: 0,
        }
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    /// Between half and all of `initial * 2^attempts`, capped at `max`.
    pub fn next_delay(&mut self) -> Duration {
        let factor = 1u32.checked_shl(self.attempts.min(20)).unwrap_or(u32::MAX);
        let base = self.initial.saturating_mul(factor).min(self.max);
        self.attempts = self.attempts.saturating_add(1);
        let half = base / 2;
        let jitter = random_below(half.as_millis() as u64 + 1);
        half + Duration::from_millis(jitter)
    }
}

fn random_below(bound: u64) -> u64 {
    let mut bytes = [0u8; 8];
    if getrandom::fill(&mut bytes).is_err() || bound == 0 {
        return 0;
    }
    u64::from_le_bytes(bytes) % bound
}

#[cfg(test)]
mod tests {
    use super::*;

    fn after(current: BotState, hook: Hook) -> Option<BotState> {
        state_after_hook(current, &hook)
    }

    #[test]
    fn hooks_move_the_bot_through_its_states() {
        use BotState::*;
        assert_eq!(
            after(Launching, Hook::SessionStart { inbox: None }),
            Some(Idle)
        );
        assert_eq!(after(Idle, Hook::PromptSubmit), Some(Busy));
        assert_eq!(after(Busy, Hook::Stop), Some(Idle));
        let permission = Hook::Notification {
            kind: "permission_prompt".into(),
        };
        assert_eq!(after(Busy, permission), Some(NeedsApproval));
        let idle_prompt = Hook::Notification {
            kind: "idle_prompt".into(),
        };
        assert_eq!(after(Idle, idle_prompt), None);
        assert_eq!(after(Idle, Hook::SessionEnd), None);
        assert_eq!(after(Idle, Hook::Stop), None, "no change, no event");
    }

    #[test]
    fn stop_failures_map_to_their_states() {
        use BotState::*;
        let failure = |error: &str| Hook::StopFailure {
            error: error.into(),
        };
        assert_eq!(after(Busy, failure("rate_limit")), Some(RateLimited));
        assert_eq!(
            after(Busy, failure("authentication_failed")),
            Some(AuthError)
        );
        assert_eq!(after(Busy, failure("server_error")), Some(Idle));
    }

    #[test]
    fn backoff_grows_with_jitter_up_to_the_cap_and_resets() {
        let mut backoff = Backoff::new(Duration::from_secs(1), Duration::from_secs(10));
        let bounds = [
            (500, 1000),
            (1000, 2000),
            (2000, 4000),
            (4000, 8000),
            (5000, 10000),
        ];
        for (low, high) in bounds {
            let delay = backoff.next_delay().as_millis();
            assert!(
                (low..=high).contains(&delay),
                "{delay} not in {low}..={high}"
            );
        }
        assert!(backoff.next_delay() <= Duration::from_secs(10));
        backoff.reset();
        assert!(backoff.next_delay() <= Duration::from_secs(1));
    }

    #[test]
    fn inbox_debug_hides_the_token() {
        let inbox = Inbox {
            socket: r"\\.\pipe\x".into(),
            token: "secret".into(),
        };
        assert!(!format!("{inbox:?}").contains("secret"));
    }
}
