//! The outline in the bot's color around the window a bot is using (spec
//! 24.9): a window of its own, on a thread of its own, the size of the
//! app's frame and a ring wider, see-through inside and clicked through.
//! It sits right above the app's window in the order of windows, not on
//! top of everything: a window of the owner's over the app covers the
//! outline too. Never in the list of windows (a tool window), never taking
//! the focus, never in a picture of the app (`PrintWindow` is only of it).
//! The bot's cursor is drawn in it too (`outline_cursor.rs`).

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock, PoisonError};

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DisableProcessWindowsGhosting, DispatchMessageW, GW_HWNDPREV,
    GetClientRect, GetMessageW, GetWindow, HWND_TOP, IsIconic, IsWindow, IsWindowVisible,
    KillTimer, LWA_COLORKEY, MSG, PostMessageW, RegisterClassW, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SetLayeredWindowAttributes, SetTimer, SetWindowDisplayAffinity, SetWindowPos,
    ShowWindow, TranslateMessage, WDA_EXCLUDEFROMCAPTURE, WM_APP, WM_PAINT, WM_TIMER, WNDCLASSW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::super::ScreenCursor;
use super::{capture, handle, outline_cursor};

/// The timer that draws the cursor while it moves.
const MOVING: usize = 1;
/// How often it draws then, about 60 times a second.
const FRAME_MS: u32 = 16;

/// Painted, then left out: the window's see-through color.
const SEE_THROUGH: COLORREF = COLORREF(0x00FF_00FF);
const SHOW: u32 = WM_APP + 1;
const HIDE: u32 = WM_APP + 2;
/// The ring's width, at 96 DPI.
const RING: i32 = 3;

/// Which window, in what color, with the bot's cursor where.
#[derive(Clone, PartialEq)]
struct Outlining {
    over: u64,
    color: (u8, u8, u8),
    cursor: Option<ScreenCursor>,
}

static OUTLINING: Mutex<Option<Outlining>> = Mutex::new(None);
/// Where it was last put and in what color, to repaint only on a change.
type Placed = ([i32; 4], (u8, u8, u8));
static PLACED: Mutex<Option<Placed>> = Mutex::new(None);
static WINDOW: OnceLock<Option<isize>> = OnceLock::new();

fn colorref((red, green, blue): (u8, u8, u8)) -> COLORREF {
    COLORREF(u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16))
}

/// The outline's window, made on its thread the first time.
fn window() -> Option<HWND> {
    let raw = WINDOW.get_or_init(|| {
        let (made, window) = mpsc::channel::<Option<isize>>();
        let started = std::thread::Builder::new()
            .name("desktop-outline".to_owned())
            .spawn(move || run(&made));
        if started.is_err() {
            return None;
        }
        window.recv().ok().flatten()
    });
    raw.map(|raw| HWND(raw as *mut std::ffi::c_void))
}

fn run(made: &Sender<Option<isize>>) {
    // SAFETY: a window class, a window and its message loop, on this thread.
    unsafe {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        // A busy outline must never turn into Windows' gray "not
        // responding" ghost over the owner's screen.
        DisableProcessWindowsGhosting();
        let Ok(module) = GetModuleHandleW(None) else {
            let _ = made.send(None);
            return;
        };
        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: module.into(),
            lpszClassName: w!("BotloftDesktopOutline"),
            ..Default::default()
        };
        RegisterClassW(&class);
        let created = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("BotloftDesktopOutline"),
            PCWSTR::null(),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            None,
            None,
        );
        let Ok(hwnd) = created else {
            let _ = made.send(None);
            return;
        };
        let _ = SetLayeredWindowAttributes(hwnd, SEE_THROUGH, 255, LWA_COLORKEY);
        // Never in a picture of the whole screen (spec 24.4).
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
        let _ = made.send(Some(hwnd.0 as isize));
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: calls on this thread's own window.
    unsafe {
        match message {
            SHOW => {
                let outlining = OUTLINING
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone();
                match outlining {
                    Some(outlining) => {
                        place(hwnd, &outlining);
                        if outline_cursor::follow(outlining.cursor.as_ref()) {
                            let _ = InvalidateRect(Some(hwnd), None, false);
                            SetTimer(Some(hwnd), MOVING, FRAME_MS, None);
                        }
                    }
                    None => hidden(hwnd),
                }
                LRESULT(0)
            }
            WM_TIMER => {
                let _ = InvalidateRect(Some(hwnd), None, false);
                if !outline_cursor::moving(std::time::Instant::now()) {
                    let _ = KillTimer(Some(hwnd), MOVING);
                }
                LRESULT(0)
            }
            HIDE => {
                hidden(hwnd);
                LRESULT(0)
            }
            WM_PAINT => {
                paint(hwnd);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

unsafe fn hidden(hwnd: HWND) {
    *PLACED.lock().unwrap_or_else(PoisonError::into_inner) = None;
    outline_cursor::follow(None);
    // SAFETY: hides this thread's own window.
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

/// Around the app's frame, right above it in the order of windows; hidden
/// while the app is minimized, hidden or gone.
unsafe fn place(hwnd: HWND, outlining: &Outlining) {
    let app = handle(outlining.over);
    // SAFETY: plain queries on a window handle.
    let showing = unsafe {
        IsWindow(Some(app)).as_bool() && IsWindowVisible(app).as_bool() && !IsIconic(app).as_bool()
    };
    let Some([left, top, width, height]) = capture::frame(outlining.over).ok().filter(|_| showing)
    else {
        // SAFETY: this thread's own window.
        unsafe { hidden(hwnd) };
        return;
    };
    // SAFETY: plain queries and a move of this thread's own window.
    unsafe {
        let ring = RING * GetDpiForWindow(app).max(96) as i32 / 96;
        let area = [left - ring, top - ring, width + 2 * ring, height + 2 * ring];
        // Right above the app: behind the window that is above it now.
        let above = GetWindow(app, GW_HWNDPREV).ok();
        let after = match above {
            Some(above) if above == hwnd => None,
            Some(above) => Some(above),
            None => Some(HWND_TOP),
        };
        let mut placed = PLACED.lock().unwrap_or_else(PoisonError::into_inner);
        let changed = *placed != Some((area, outlining.color));
        *placed = Some((area, outlining.color));
        drop(placed);
        let mut flags = SWP_NOACTIVATE;
        if after.is_none() {
            flags |= windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER;
        }
        let _ = SetWindowPos(hwnd, after, area[0], area[1], area[2], area[3], flags);
        if changed {
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

unsafe fn paint(hwnd: HWND) {
    let outlining = OUTLINING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let color = outlining
        .as_ref()
        .map_or((0xFF, 0x7A, 0x59), |outlining| outlining.color);
    // SAFETY: painting this thread's own window between Begin and EndPaint,
    // every object made here freed before the end.
    unsafe {
        let mut paint = PAINTSTRUCT::default();
        let dc = BeginPaint(hwnd, &mut paint);
        let mut area = RECT::default();
        let _ = GetClientRect(hwnd, &mut area);
        let clear = CreateSolidBrush(SEE_THROUGH);
        FillRect(dc, &area, clear);
        let brush = CreateSolidBrush(colorref(color));
        let ring = RING * GetDpiForWindow(hwnd).max(96) as i32 / 96;
        for edge in [
            RECT {
                bottom: area.top + ring,
                ..area
            },
            RECT {
                top: area.bottom - ring,
                ..area
            },
            RECT {
                right: area.left + ring,
                ..area
            },
            RECT {
                left: area.right - ring,
                ..area
            },
        ] {
            FillRect(dc, &edge, brush);
        }
        if let Some(cursor) = outlining
            .as_ref()
            .and_then(|outlining| outlining.cursor.as_ref())
        {
            let dpi = GetDpiForWindow(hwnd).max(96) as i32;
            let inside = RECT {
                left: area.left + ring,
                top: area.top + ring,
                right: area.right - ring,
                bottom: area.bottom - ring,
            };
            outline_cursor::paint(dc, inside, |length| length * dpi / 96, color, cursor);
        }
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(clear.into());
        let _ = EndPaint(hwnd, &paint);
    }
}

/// Outlines window `over` in `color`, with the bot's `cursor`, or puts the
/// outline where the window is now.
pub fn show(over: u64, color: (u8, u8, u8), cursor: Option<ScreenCursor>) {
    *OUTLINING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Outlining {
        over,
        color,
        cursor,
    });
    if let Some(hwnd) = window() {
        // SAFETY: asks the outline's own thread to place it.
        let _ = unsafe { PostMessageW(Some(hwnd), SHOW, WPARAM(0), LPARAM(0)) };
    }
}

/// Takes the outline away.
pub fn hide() {
    *OUTLINING.lock().unwrap_or_else(PoisonError::into_inner) = None;
    if let Some(Some(raw)) = WINDOW.get() {
        let hwnd = HWND(*raw as *mut std::ffi::c_void);
        // SAFETY: asks the outline's own thread to hide it.
        let _ = unsafe { PostMessageW(Some(hwnd), HIDE, WPARAM(0), LPARAM(0)) };
    }
}
