//! With real windows: a window of the test's own, painted red, found in
//! the list, pictured, minimized and closed.

use std::sync::Once;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    BM_SETCHECK, BS_AUTOCHECKBOX, BS_PUSHBUTTON, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DispatchMessageW, ES_PASSWORD, GetMessageW, HMENU, MSG, PostMessageW, PostQuitMessage,
    RegisterClassW, SW_MINIMIZE, SendMessageW, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

use super::super::{DesktopError, never, read, render, windows as listed};
use super::capture;

const CLASS: PCWSTR = w!("BotloftDesktopTest");

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_DESTROY {
        // SAFETY: ends this thread's message loop.
        unsafe { PostQuitMessage(0) };
        return LRESULT(0);
    }
    // SAFETY: everything else as Windows does by default.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// A button, a field with text, a password field and a ticked check box,
/// on the left of the window: its middle stays red.
///
/// # Safety
///
/// `parent` is a window of this thread.
unsafe fn controls(parent: HWND) {
    let child = |class: PCWSTR, text: PCWSTR, style: u32, top: i32, width: i32| {
        // SAFETY: a child of `parent`, on its thread.
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                text,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(style),
                10,
                top,
                width,
                24,
                Some(parent),
                Some(HMENU(std::ptr::null_mut())),
                None,
                None,
            )
            .expect("a control")
        }
    };
    child(w!("BUTTON"), w!("Save"), BS_PUSHBUTTON as u32, 10, 120);
    child(w!("EDIT"), w!("hello"), WS_BORDER.0, 40, 160);
    child(
        w!("EDIT"),
        w!("secret"),
        WS_BORDER.0 | ES_PASSWORD as u32,
        70,
        160,
    );
    let check = child(
        w!("BUTTON"),
        w!("Remember"),
        BS_AUTOCHECKBOX as u32,
        100,
        160,
    );
    // SAFETY: ticks our own check box.
    unsafe { SendMessageW(check, BM_SETCHECK, Some(WPARAM(1)), None) };
}

/// A red window of the test's own, on a thread with its message loop.
struct TestWindow {
    id: u64,
    thread: Option<JoinHandle<()>>,
}

impl TestWindow {
    fn open(title: &str) -> Self {
        static CLASS_ONCE: Once = Once::new();
        CLASS_ONCE.call_once(|| {
            // SAFETY: a window class with a static name and procedure.
            unsafe {
                let class = WNDCLASSW {
                    lpfnWndProc: Some(procedure),
                    hInstance: GetModuleHandleW(None).expect("module").into(),
                    lpszClassName: CLASS,
                    // Pure red: 0x00BBGGRR.
                    hbrBackground: HBRUSH(CreateSolidBrush(COLORREF(0x0000_00FF)).0),
                    ..Default::default()
                };
                assert_ne!(RegisterClassW(&class), 0, "register the class");
            }
        });
        let title: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
        let (sent, opened) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            // SAFETY: a window on this thread, with its message loop.
            unsafe {
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    CLASS,
                    PCWSTR(title.as_ptr()),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    420,
                    320,
                    None,
                    None,
                    None,
                    None,
                )
                .expect("create the window");
                controls(hwnd);
                sent.send(hwnd.0 as usize as u64).expect("send");
                let mut message = MSG::default();
                while GetMessageW(&mut message, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        });
        let id = opened
            .recv_timeout(Duration::from_secs(5))
            .expect("the window opened");
        // Time to paint.
        std::thread::sleep(Duration::from_millis(300));
        Self {
            id,
            thread: Some(thread),
        }
    }

    fn close(&mut self) {
        // SAFETY: asks the window's own thread to close it.
        let _ =
            unsafe { PostMessageW(Some(super::handle(self.id)), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            thread.join().expect("the window's thread");
        }
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        self.close();
    }
}

#[test]
fn the_list_has_the_window_with_its_app() {
    let window = TestWindow::open("Botloft desktop test: list");
    let found = listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .expect("our window is in the list");
    assert_eq!(found.title, "Botloft desktop test: list");
    assert_eq!(found.class, "BotloftDesktopTest");
    let exe = std::env::current_exe().expect("exe");
    assert_eq!(
        found.app.path.to_string_lossy().to_lowercase(),
        exe.to_string_lossy().to_lowercase()
    );
    assert!(!found.app.name.is_empty());
    assert!(!found.minimized);
    assert!(!found.elevated, "the test is not running as administrator");
    assert_eq!(never(&found), None);
}

#[test]
fn a_picture_is_only_the_window_in_its_own_pixels() {
    let window = TestWindow::open("Botloft desktop test: picture");
    let (rgb, width, height, scale) = capture(window.id).expect("picture");
    assert!(scale >= 1.0);
    // 420 × 320 logical pixels, the title bar included, without the
    // invisible border: within a few pixels of it at any display scale.
    let logical = |pixels: u32| f64::from(pixels) / scale;
    assert!((logical(width) - 420.0).abs() < 20.0, "{width} at {scale}");
    assert!(
        (logical(height) - 320.0).abs() < 20.0,
        "{height} at {scale}"
    );
    // The middle of the window is its red background.
    let middle = ((height / 2 * width + width / 2) * 3) as usize;
    assert_eq!(&rgb[middle..middle + 3], [255, 0, 0]);
    let picture = super::super::picture(window.id).expect("jpeg");
    assert_eq!(&picture.jpeg[..2], [0xFF, 0xD8]);
}

#[test]
fn a_minimized_window_is_listed_but_has_nothing_to_see_and_a_closed_one_is_gone() {
    let mut window = TestWindow::open("Botloft desktop test: minimized");
    // SAFETY: a plain request on our own window.
    let _ = unsafe { ShowWindow(super::handle(window.id), SW_MINIMIZE) };
    std::thread::sleep(Duration::from_millis(300));
    let found = listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .expect("still listed");
    assert!(found.minimized);
    assert!(matches!(capture(window.id), Err(DesktopError::Minimized)));
    let id = window.id;
    window.close();
    assert!(matches!(capture(id), Err(DesktopError::Gone)));
    assert!(
        listed()
            .expect("windows")
            .iter()
            .all(|found| found.id != id)
    );
}

#[test]
fn a_window_reads_as_its_controls_and_never_shows_a_password() {
    let window = TestWindow::open("Botloft desktop test: read");
    let controls = read(window.id).expect("read");
    let (text, next) = render(&controls, 0);
    assert_eq!(next, None);
    assert!(text.contains(r#"button "Save""#), "{text}");
    assert!(text.contains(r#"= "hello""#), "{text}");
    assert!(text.contains(r#"check box "Remember" (checked)"#), "{text}");
    assert!(text.contains("the owner types it"), "{text}");
    assert!(!text.contains("secret"), "{text}");
    let field = controls
        .iter()
        .find(|control| control.value.as_deref() == Some("hello"))
        .expect("the field");
    assert!(!field.runtime_id.is_empty());
    // A window that closed has nothing to read.
    let id = window.id;
    drop(window);
    assert!(read(id).is_err());
}
