//! Per-bot supervision state and restart backoff (spec 7).

use std::time::Duration;

use botloft_core::protocol::{BotState, PermissionMode};
use tokio::time::Instant;

use crate::runtime::ProcessControl;

pub(super) struct Slot {
    pub state: BotState,
    /// Generation of the current or last process.
    pub generation: Option<u64>,
    pub running: Option<Running>,
    pub backoff: Backoff,
    pub restart_at: Option<Instant>,
    /// Start the next process with a new conversation.
    pub fresh_next: bool,
    /// Why the running process is being killed, if the daemon asked.
    pub stop: Option<StopIntent>,
    /// Messages written to the process whose turn has not ended yet.
    pub turns: u32,
    /// Permission requests waiting for the owner.
    pub approvals: u32,
    /// While `rate_limited`: Unix ms when the limit resets.
    pub limited_until: Option<i64>,
    /// A new permission mode or model waits for the running turn to end
    /// (spec 7.4).
    pub restart_when_idle: bool,
}

impl Slot {
    pub fn new(backoff: Backoff) -> Self {
        Self {
            state: BotState::Offline,
            generation: None,
            running: None,
            backoff,
            restart_at: None,
            fresh_next: false,
            stop: None,
            turns: 0,
            approvals: 0,
            limited_until: None,
            restart_when_idle: false,
        }
    }

    /// The state that follows from the counters once nothing blocks the bot.
    pub fn working_state(&self) -> BotState {
        if self.approvals > 0 {
            BotState::NeedsApproval
        } else if self.turns > 0 {
            BotState::Busy
        } else {
            BotState::Idle
        }
    }

    /// Whether the counters decide the state, rather than a launch, a
    /// limit, a sign-in problem or a stop.
    pub fn is_working(&self) -> bool {
        matches!(
            self.state,
            BotState::Idle | BotState::Busy | BotState::NeedsApproval
        )
    }
}

pub(super) struct Running {
    pub control: Box<dyn ProcessControl>,
    pub started: Instant,
    /// Started with `--resume`.
    pub resumed: bool,
    pub token_hash: String,
    /// The `--permission-mode` it started with.
    pub permission_mode: PermissionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StopIntent {
    Restart {
        fresh: bool,
    },
    /// Paused, archived or signed out: settle in this state.
    Halt(BotState),
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

    #[test]
    fn counters_decide_the_working_state() {
        let mut slot = Slot::new(Backoff::new(Duration::ZERO, Duration::ZERO));
        assert_eq!(slot.working_state(), BotState::Idle);
        slot.turns = 2;
        assert_eq!(slot.working_state(), BotState::Busy);
        slot.approvals = 1;
        assert_eq!(slot.working_state(), BotState::NeedsApproval);
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
}
