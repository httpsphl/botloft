//! The notice on the owner's screen while a bot uses their real mouse and
//! keyboard (spec 24.7): a thin border in the bot's color around the
//! monitor the app is on, and a pill at its top with what is happening and
//! how to stop it. A window of its own, on a thread of its own: always on
//! top, clicked through, never taking the focus, never in the list of
//! windows (a tool window), never in a picture of another window.

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock, PoisonError};

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreatePen, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
    DeleteObject, DrawTextW, EndPaint, FillRect, GetMonitorInfoW, HGDIOBJ, InvalidateRect,
    MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, PAINTSTRUCT, PS_SOLID, RoundRect,
    SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, HWND_TOPMOST, LWA_COLORKEY,
    MSG, PostMessageW, RegisterClassW, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
    SetLayeredWindowAttributes, SetWindowPos, ShowWindow, TranslateMessage, WM_APP, WM_PAINT,
    WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
    WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::handle;

/// Painted, then left out: the window's see-through color.
const SEE_THROUGH: COLORREF = COLORREF(0x00FF_00FF);
const SHOW: u32 = WM_APP + 1;
const HIDE: u32 = WM_APP + 2;
/// The border's width and the pill's size, at 96 DPI.
const BORDER: i32 = 4;
const PILL_HEIGHT: i32 = 36;
const PILL_MARGIN: i32 = 16;

/// What the notice says, in what color, over which window's monitor.
#[derive(Clone)]
struct Showing {
    text: String,
    color: (u8, u8, u8),
    over: u64,
}

static SHOWING: Mutex<Option<Showing>> = Mutex::new(None);
static WINDOW: OnceLock<Option<isize>> = OnceLock::new();

fn colorref((red, green, blue): (u8, u8, u8)) -> COLORREF {
    COLORREF(u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16))
}

/// The notice's window, made on its thread the first time.
fn window() -> Option<HWND> {
    let raw = WINDOW.get_or_init(|| {
        let (made, window) = mpsc::channel::<Option<isize>>();
        let started = std::thread::Builder::new()
            .name("desktop-notice".to_owned())
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
        let Ok(module) = GetModuleHandleW(None) else {
            let _ = made.send(None);
            return;
        };
        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: module.into(),
            lpszClassName: w!("BotloftDesktopNotice"),
            ..Default::default()
        };
        RegisterClassW(&class);
        let created = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("BotloftDesktopNotice"),
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
                let showing = SHOWING
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .clone();
                if let Some(showing) = showing {
                    place(hwnd, showing.over);
                    let _ = InvalidateRect(Some(hwnd), None, true);
                    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                }
                LRESULT(0)
            }
            HIDE => {
                let _ = ShowWindow(hwnd, SW_HIDE);
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

/// Over the whole monitor the bot's window is on.
unsafe fn place(hwnd: HWND, over: u64) {
    // SAFETY: plain queries and a move of this thread's own window.
    unsafe {
        let monitor = MonitorFromWindow(handle(over), MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return;
        }
        let area = info.rcMonitor;
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            area.left,
            area.top,
            area.right - area.left,
            area.bottom - area.top,
            SWP_NOACTIVATE,
        );
    }
}

unsafe fn paint(hwnd: HWND) {
    let Some(showing) = SHOWING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
    else {
        return;
    };
    // SAFETY: painting this thread's own window between Begin and EndPaint,
    // every object made here freed before the end.
    unsafe {
        let mut paint = PAINTSTRUCT::default();
        let dc = BeginPaint(hwnd, &mut paint);
        let mut area = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut area);
        let scale = |length: i32| length * GetDpiForWindow(hwnd).max(96) as i32 / 96;
        let clear = CreateSolidBrush(SEE_THROUGH);
        FillRect(dc, &area, clear);
        let color = CreateSolidBrush(colorref(showing.color));
        let border = scale(BORDER);
        for edge in [
            RECT {
                bottom: area.top + border,
                ..area
            },
            RECT {
                top: area.bottom - border,
                ..area
            },
            RECT {
                right: area.left + border,
                ..area
            },
            RECT {
                left: area.right - border,
                ..area
            },
        ] {
            FillRect(dc, &edge, color);
        }
        // The pill, centered at the top, dark with the bot's color around.
        let text: Vec<u16> = showing.text.encode_utf16().collect();
        let width =
            scale(14 + 8 * text.len().min(90) as i32).min(area.right - area.left - 2 * border);
        let left = (area.right - area.left - width) / 2;
        let top = border + scale(PILL_MARGIN);
        let pen = CreatePen(PS_SOLID, scale(2), colorref(showing.color));
        let fill = CreateSolidBrush(COLORREF(0x001B_1B1B));
        let old_pen: HGDIOBJ = SelectObject(dc, pen.into());
        let old_brush: HGDIOBJ = SelectObject(dc, fill.into());
        let pill = scale(PILL_HEIGHT);
        let _ = RoundRect(dc, left, top, left + width, top + pill, pill, pill);
        let font = CreateFontW(
            -scale(15),
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            0,
            w!("Segoe UI"),
        );
        let old_font = SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0x00FF_FFFF));
        let mut place = RECT {
            left,
            top,
            right: left + width,
            bottom: top + pill,
        };
        let mut text = text;
        DrawTextW(
            dc,
            &mut text,
            &mut place,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(dc, old_font);
        SelectObject(dc, old_pen);
        SelectObject(dc, old_brush);
        for object in [
            font.into(),
            pen.into(),
            fill.into(),
            color.into(),
            clear.into(),
        ] {
            let _ = DeleteObject(object);
        }
        let _ = EndPaint(hwnd, &paint);
    }
}

/// Shows `text` with a border in `color` over the monitor of window
/// `over`, or changes what it shows.
pub fn show(text: &str, color: (u8, u8, u8), over: u64) {
    *SHOWING.lock().unwrap_or_else(PoisonError::into_inner) = Some(Showing {
        text: text.to_owned(),
        color,
        over,
    });
    if let Some(hwnd) = window() {
        // SAFETY: asks the notice's own thread to show it.
        let _ = unsafe { PostMessageW(Some(hwnd), SHOW, WPARAM(0), LPARAM(0)) };
    }
}

/// Takes the notice away.
pub fn hide() {
    *SHOWING.lock().unwrap_or_else(PoisonError::into_inner) = None;
    if let Some(Some(raw)) = WINDOW.get() {
        let hwnd = HWND(*raw as *mut std::ffi::c_void);
        // SAFETY: asks the notice's own thread to hide it.
        let _ = unsafe { PostMessageW(Some(hwnd), HIDE, WPARAM(0), LPARAM(0)) };
    }
}
