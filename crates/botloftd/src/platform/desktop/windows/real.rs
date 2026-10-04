//! The real mouse and keyboard (spec 24.7), for what accessibility cannot
//! do: `SendInput`, with the app's window brought to the front first.
//! Before every step the window must still be in front, a click must land
//! on it and not on another window over it, and the owner must not have
//! touched the mouse or keyboard: then it stops at once.

use std::sync::atomic::Ordering;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, LASTINPUTINFO, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK,
    MOUSEINPUT, SendInput, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE,
    VK_F1, VK_HOME, VK_INSERT, VK_LEFT, VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT,
    VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetForegroundWindow, GetSystemMetrics, IsIconic, IsWindow,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_RESTORE,
    SetForegroundWindow, ShowWindow, WindowFromPoint,
};

use super::super::keys::{Key, Keys};
use super::super::{DesktopError, Spot};
use super::handle;
use super::owner::{BOT_TICK, OWNER_TICK, OwnerWatch};

/// Time for the window to come to the front, and between steps.
const BREATH: Duration = Duration::from_millis(60);
/// Characters typed between checks.
const CHUNK: usize = 8;

fn last_input() -> u32 {
    let mut last = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    // SAFETY: `last` has its size set.
    let _ = unsafe { GetLastInputInfo(&mut last) };
    last.dwTime
}

/// One real action: the window in front, the owner watched, the input
/// sent in steps, each checked first.
struct Hands {
    hwnd: HWND,
    owner: OwnerWatch,
}

impl Hands {
    fn take(id: u64) -> Result<Self, DesktopError> {
        let hwnd = handle(id);
        // SAFETY: plain calls on a window handle.
        unsafe {
            if !IsWindow(Some(hwnd)).as_bool() {
                return Err(DesktopError::Gone);
            }
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        // The owner's last input before the bot's, for their idle time.
        if BOT_TICK.load(Ordering::SeqCst) != last_input() {
            OWNER_TICK.store(last_input(), Ordering::SeqCst);
        }
        let owner = OwnerWatch::start()?;
        let hands = Self { hwnd, owner };
        hands.front()?;
        Ok(hands)
    }

    /// Brings the window to the front: Windows lets a process do that when
    /// it sent the last input, so an empty mouse move goes first.
    fn front(&self) -> Result<(), DesktopError> {
        // SAFETY: plain calls on a window handle.
        unsafe {
            if IsIconic(self.hwnd).as_bool() {
                let _ = ShowWindow(self.hwnd, SW_RESTORE);
            }
            if GetForegroundWindow() != self.hwnd {
                send(&[mouse_input(0, 0, MOUSEEVENTF_MOVE)])?;
                let _ = SetForegroundWindow(self.hwnd);
                std::thread::sleep(BREATH);
            }
        }
        self.check()
    }

    /// The window is in front and the owner did not move.
    fn check(&self) -> Result<(), DesktopError> {
        if self.owner.moved() {
            return Err(DesktopError::OwnerTookOver);
        }
        // SAFETY: a plain query.
        let front = unsafe { GetForegroundWindow() };
        if front.is_invalid() {
            return Err(DesktopError::Locked);
        }
        // SAFETY: a plain query on a window handle.
        if unsafe { GetAncestor(front, GA_ROOT) } != self.hwnd {
            return Err(DesktopError::NotInFront);
        }
        Ok(())
    }

    fn sent(&self, inputs: &[INPUT]) -> Result<(), DesktopError> {
        self.check()?;
        send(inputs)?;
        BOT_TICK.store(last_input(), Ordering::SeqCst);
        Ok(())
    }

    /// The point on the screen, in real pixels, of `spot` in the window.
    fn screen(&self, spot: Spot) -> Result<POINT, DesktopError> {
        match spot {
            Spot::Screen { x, y } => Ok(POINT { x, y }),
            Spot::Window { x, y } => {
                let mut frame = RECT::default();
                // SAFETY: `frame` is a RECT of the size given.
                unsafe {
                    DwmGetWindowAttribute(
                        self.hwnd,
                        DWMWA_EXTENDED_FRAME_BOUNDS,
                        std::ptr::from_mut(&mut frame).cast(),
                        size_of::<RECT>() as u32,
                    )
                }
                .map_err(|_| DesktopError::Gone)?;
                let width = f64::from(frame.right - frame.left);
                let height = f64::from(frame.bottom - frame.top);
                Ok(POINT {
                    x: frame.left + (x.clamp(0.0, 1.0) * width) as i32,
                    y: frame.top + (y.clamp(0.0, 1.0) * height) as i32,
                })
            }
        }
    }
}

fn send(inputs: &[INPUT]) -> Result<(), DesktopError> {
    // SAFETY: the inputs are well formed and live through the call.
    let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(DesktopError::Locked);
    }
    Ok(())
}

fn mouse_input(x: i32, y: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

fn key_input(key: VIRTUAL_KEY, unicode: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: unicode,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

/// Clicks `spot` in window `id` with the real mouse.
pub fn click(id: u64, spot: Spot) -> Result<(), DesktopError> {
    let hands = Hands::take(id)?;
    let point = hands.screen(spot)?;
    // SAFETY: a plain query.
    let under = unsafe { GetAncestor(WindowFromPoint(point), GA_ROOT) };
    if under != hands.hwnd {
        return Err(DesktopError::Covered);
    }
    // Absolute moves go from 0 to 65535 across all the screens.
    // SAFETY: plain queries.
    let (left, top, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1),
            GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1),
        )
    };
    let x = ((i64::from(point.x - left) * 65535) / i64::from(width)) as i32;
    let y = ((i64::from(point.y - top) * 65535) / i64::from(height)) as i32;
    let moved = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    hands.sent(&[mouse_input(x, y, moved)])?;
    std::thread::sleep(BREATH);
    hands.sent(&[
        mouse_input(0, 0, MOUSEEVENTF_LEFTDOWN),
        mouse_input(0, 0, MOUSEEVENTF_LEFTUP),
    ])
}

/// Types `text` into window `id`, where its focus is, with the real
/// keyboard.
pub fn type_text(id: u64, text: &str) -> Result<(), DesktopError> {
    let hands = Hands::take(id)?;
    let units: Vec<u16> = text.replace("\r\n", "\n").encode_utf16().collect();
    for chunk in units.chunks(CHUNK) {
        let mut inputs = Vec::with_capacity(chunk.len() * 2);
        for &unit in chunk {
            if unit == u16::from(b'\n') {
                inputs.push(key_input(VK_RETURN, 0, KEYBD_EVENT_FLAGS(0)));
                inputs.push(key_input(VK_RETURN, 0, KEYEVENTF_KEYUP));
            } else {
                inputs.push(key_input(VIRTUAL_KEY(0), unit, KEYEVENTF_UNICODE));
                inputs.push(key_input(
                    VIRTUAL_KEY(0),
                    unit,
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                ));
            }
        }
        hands.sent(&inputs)?;
    }
    Ok(())
}

fn virtual_key(key: Key) -> VIRTUAL_KEY {
    match key {
        Key::Enter => VK_RETURN,
        Key::Tab => VK_TAB,
        Key::Escape => VK_ESCAPE,
        Key::Backspace => VK_BACK,
        Key::Delete => VK_DELETE,
        Key::Space => VK_SPACE,
        Key::Up => VK_UP,
        Key::Down => VK_DOWN,
        Key::Left => VK_LEFT,
        Key::Right => VK_RIGHT,
        Key::Home => VK_HOME,
        Key::End => VK_END,
        Key::PageUp => VK_PRIOR,
        Key::PageDown => VK_NEXT,
        Key::Insert => VK_INSERT,
        Key::F(number) => VIRTUAL_KEY(VK_F1.0 + u16::from(number) - 1),
        // Letters and digits have their ASCII code as virtual key.
        Key::Char(char) => VIRTUAL_KEY(char as u16),
    }
}

/// Presses `keys` in window `id` with the real keyboard: the modifiers
/// down, the key, all up again.
pub fn press(id: u64, keys: &Keys) -> Result<(), DesktopError> {
    let hands = Hands::take(id)?;
    let Some(key) = keys.key else {
        return Ok(());
    };
    let held: Vec<VIRTUAL_KEY> = [
        (keys.ctrl, VK_CONTROL),
        (keys.alt, VK_MENU),
        (keys.shift, VK_SHIFT),
    ]
    .into_iter()
    .filter_map(|(on, key)| on.then_some(key))
    .collect();
    let mut inputs: Vec<INPUT> = held
        .iter()
        .map(|&key| key_input(key, 0, KEYBD_EVENT_FLAGS(0)))
        .collect();
    inputs.push(key_input(virtual_key(key), 0, KEYBD_EVENT_FLAGS(0)));
    inputs.push(key_input(virtual_key(key), 0, KEYEVENTF_KEYUP));
    inputs.extend(
        held.iter()
            .rev()
            .map(|&key| key_input(key, 0, KEYEVENTF_KEYUP)),
    );
    hands.sent(&inputs)
}
