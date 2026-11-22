//! How much memory the machine has for agents (spec 42.4). An awake agent
//! is a Claude Code process of some 230 MB, plus its tools: before starting
//! one, the supervisor reads what is free, so a small server queues the
//! work instead of running out of memory and having its processes killed.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const MB: u64 = 1024 * 1024;

/// Room one awake agent takes: 228 MB measured after a turn (spec 42.4),
/// with some for its tools.
pub const PER_AGENT: u64 = 300 * MB;
/// What stays free for the system and the daemon.
pub const HEADROOM: u64 = 256 * MB;
/// Up to this much memory, the machine is small.
pub const SMALL_MACHINE: u64 = 2 * 1024 * MB;
/// On a small machine, an idle agent sleeps this soon.
pub const SMALL_MACHINE_NAP: Duration = Duration::from_secs(2 * 60);

/// What the machine has, in bytes: for the daemon, the lower of the system's
/// and its control group's limit (a service or a container).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
}

impl MemoryInfo {
    /// How many more agents fit now.
    pub fn room(&self) -> usize {
        usize::try_from(self.available.saturating_sub(HEADROOM) / PER_AGENT).unwrap_or(usize::MAX)
    }

    pub fn small(&self) -> bool {
        self.total <= SMALL_MACHINE
    }

    /// Less than half is free.
    pub fn pressed(&self) -> bool {
        self.available < self.total / 2
    }
}

/// Reads the machine's memory; tests give their own.
pub trait MemoryProbe: Send + Sync {
    /// `None` when the system cannot say: then nothing is limited by it.
    fn read(&self) -> Option<MemoryInfo>;

    /// How soon an idle agent sleeps on a small machine.
    fn small_machine_nap(&self) -> Duration {
        SMALL_MACHINE_NAP
    }
}

/// The system's own numbers.
pub struct SystemMemory;

impl MemoryProbe for SystemMemory {
    fn read(&self) -> Option<MemoryInfo> {
        crate::platform::memory()
    }
}

/// For tests: the machine they say, unknown until they say one.
#[derive(Clone, Default)]
pub struct FakeMemory {
    state: Arc<Mutex<FakeState>>,
}

#[derive(Default)]
struct FakeState {
    info: Option<MemoryInfo>,
    nap: Option<Duration>,
}

impl FakeMemory {
    pub fn set(&self, info: Option<MemoryInfo>) {
        self.lock().info = info;
    }

    /// A machine of `total` bytes with room for exactly `agents` more.
    pub fn set_room(&self, total: u64, agents: u64) {
        self.set(Some(MemoryInfo {
            total,
            available: HEADROOM + agents * PER_AGENT,
        }));
    }

    /// How soon an idle agent sleeps on a small machine, shorter than the
    /// real two minutes.
    pub fn set_small_machine_nap(&self, nap: Duration) {
        self.lock().nap = Some(nap);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FakeState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl MemoryProbe for FakeMemory {
    fn read(&self) -> Option<MemoryInfo> {
        self.lock().info
    }

    fn small_machine_nap(&self) -> Duration {
        self.lock().nap.unwrap_or(SMALL_MACHINE_NAP)
    }
}

/// How long an idle agent stays awake: the owner's setting, shorter on a
/// small machine. Sleep turned off stays off.
pub fn nap(
    setting: Option<Duration>,
    memory: Option<MemoryInfo>,
    small_nap: Duration,
) -> Option<Duration> {
    match memory {
        Some(info) if info.small() => setting.map(|nap| nap.min(small_nap)),
        _ => setting,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1024 * MB;

    #[test]
    fn room_counts_whole_agents_after_the_headroom() {
        let info = |available| MemoryInfo {
            total: 8 * GB,
            available,
        };
        assert_eq!(info(HEADROOM).room(), 0);
        assert_eq!(info(HEADROOM + PER_AGENT - 1).room(), 0);
        assert_eq!(info(HEADROOM + 3 * PER_AGENT).room(), 3);
        assert_eq!(info(0).room(), 0);
    }

    #[test]
    fn a_small_machine_naps_sooner_but_sleep_off_stays_off() {
        let small = Some(MemoryInfo {
            total: GB,
            available: GB / 2,
        });
        let big = Some(MemoryInfo {
            total: 16 * GB,
            available: 8 * GB,
        });
        let hour = Some(Duration::from_secs(3600));
        assert_eq!(nap(hour, small, SMALL_MACHINE_NAP), Some(SMALL_MACHINE_NAP));
        assert_eq!(nap(hour, big, SMALL_MACHINE_NAP), hour);
        assert_eq!(nap(hour, None, SMALL_MACHINE_NAP), hour);
        assert_eq!(nap(None, small, SMALL_MACHINE_NAP), None);
        let minute = Some(Duration::from_secs(60));
        assert_eq!(nap(minute, small, SMALL_MACHINE_NAP), minute);
    }

    #[test]
    fn pressed_is_less_than_half_free() {
        assert!(
            MemoryInfo {
                total: 2 * GB,
                available: GB - 1
            }
            .pressed()
        );
        assert!(
            !MemoryInfo {
                total: 2 * GB,
                available: GB
            }
            .pressed()
        );
    }
}
