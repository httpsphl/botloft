//! Calling the island by holding the right Ctrl alone (spec 31.6). A thread
//! reads, twenty times a second, whether that one key is down, and only
//! while it is, whether any other key is too (Ctrl+C is not a call). No
//! keyboard hook: a hook would see every key the owner types. Nothing read
//! is kept or logged.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The event the island hears when the owner calls it.
pub const SUMMON_EVENT: &str = "botloft://summon";

static ON: AtomicBool = AtomicBool::new(false);
static HOLD_MS: AtomicU64 = AtomicU64::new(1000);
#[cfg(windows)]
static STARTED: std::sync::Once = std::sync::Once::new();

/// The right Ctrl held alone long enough: one call per hold.
#[derive(Debug, Default)]
pub struct Hold {
    since: Option<Instant>,
    /// Another key went down during this hold.
    spoiled: bool,
    called: bool,
}

impl Hold {
    /// Whether this reading calls the island: `down` is the right Ctrl,
    /// `others` any other key.
    pub fn step(&mut self, down: bool, others: bool, now: Instant, hold: Duration) -> bool {
        if !down {
            *self = Self::default();
            return false;
        }
        let since = *self.since.get_or_insert(now);
        self.spoiled |= others;
        if self.spoiled || self.called || now.duration_since(since) < hold {
            return false;
        }
        self.called = true;
        true
    }
}

/// Turns the call on or off, and how long the key must be held. The thread
/// starts the first time it is turned on and then only sleeps while off.
pub fn set(app: &tauri::AppHandle, on: bool, hold_ms: u64) {
    HOLD_MS.store(hold_ms.clamp(300, 5000), Ordering::Relaxed);
    ON.store(on, Ordering::Relaxed);
    #[cfg(windows)]
    if on {
        let app = app.clone();
        STARTED.call_once(move || {
            std::thread::spawn(move || watch(&app));
        });
    }
    #[cfg(not(windows))]
    let _ = app;
}

#[cfg(windows)]
fn watch(app: &tauri::AppHandle) {
    use tauri::Emitter as _;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_RCONTROL};

    // SAFETY: the call only reads a key's state; it takes no pointers.
    let pressed = |key: u16| unsafe { GetAsyncKeyState(i32::from(key)) } < 0;
    let mut hold = Hold::default();
    loop {
        if !ON.load(Ordering::Relaxed) {
            hold = Hold::default();
            std::thread::sleep(Duration::from_millis(250));
            continue;
        }
        let down = pressed(VK_RCONTROL.0);
        // The generic Ctrl is down with the right one; every other key counts.
        let others = down
            && (0x08..=0xFE)
                .filter(|key| *key != VK_CONTROL.0 && *key != VK_RCONTROL.0)
                .any(pressed);
        let hold_for = Duration::from_millis(HOLD_MS.load(Ordering::Relaxed));
        if hold.step(down, others, Instant::now(), hold_for) {
            let _ = app.emit_to("island", SUMMON_EVENT, ());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::Hold;

    const HOLD: Duration = Duration::from_secs(1);

    #[test]
    fn held_alone_long_enough_it_calls_once() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut hold = Hold::default();
        assert!(!hold.step(true, false, at(0), HOLD));
        assert!(!hold.step(true, false, at(950), HOLD));
        assert!(hold.step(true, false, at(1000), HOLD));
        // Still held: no second call until it is let go.
        assert!(!hold.step(true, false, at(3000), HOLD));
        assert!(!hold.step(false, false, at(3050), HOLD));
        assert!(!hold.step(true, false, at(3100), HOLD));
        assert!(hold.step(true, false, at(4100), HOLD));
    }

    #[test]
    fn another_key_during_the_hold_spoils_it() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut hold = Hold::default();
        assert!(!hold.step(true, false, at(0), HOLD));
        assert!(!hold.step(true, true, at(200), HOLD));
        // Ctrl+C let go of the C: the same hold still does not call.
        assert!(!hold.step(true, false, at(1500), HOLD));
        assert!(!hold.step(false, false, at(1600), HOLD));
        assert!(!hold.step(true, false, at(1700), HOLD));
        assert!(hold.step(true, false, at(2700), HOLD));
    }

    #[test]
    fn a_short_tap_does_nothing() {
        let start = Instant::now();
        let mut hold = Hold::default();
        assert!(!hold.step(true, false, start, HOLD));
        assert!(!hold.step(false, false, start + Duration::from_millis(300), HOLD));
        assert!(!hold.step(true, false, start + Duration::from_millis(1400), HOLD));
    }
}
