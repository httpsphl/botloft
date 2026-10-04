//! The desktop tests' own window (spec 24): a Save button that renames it,
//! a field and a password field, on a thread with its message loop; and a
//! daemon with a crew and a bot ready to use it. Windows only.

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    BS_PUSHBUTTON, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_PASSWORD,
    GetMessageW, HMENU, MSG, PostMessageW, PostQuitMessage, RegisterClassW, SetWindowTextW,
    TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_COMMAND, WM_DESTROY, WNDCLASSW,
    WS_BORDER, WS_CHILD, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

use super::bots::ready_bot;
use super::mcp::Mcp;
use super::{Client, TestDaemon};

pub const TITLE: &str = "Botloft desktop: the clinic";

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
    // The Save button (id 1) clicked: the window says so in its title.
    if message == WM_COMMAND && wparam.0 & 0xFFFF == 1 && wparam.0 >> 16 == 0 {
        // SAFETY: renames our own window, on its thread.
        let _ = unsafe { SetWindowTextW(hwnd, w!("Saved")) };
        return LRESULT(0);
    }
    // SAFETY: everything else as Windows does by default.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// A window of the test's own, on a thread with its message loop.
pub struct TestWindow {
    pub id: u64,
    thread: Option<JoinHandle<()>>,
}

impl TestWindow {
    pub fn open() -> Self {
        let (sent, opened) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            // SAFETY: a window and its controls on this thread, with its loop.
            unsafe {
                let class = WNDCLASSW {
                    lpfnWndProc: Some(procedure),
                    hInstance: GetModuleHandleW(None).expect("module").into(),
                    lpszClassName: w!("BotloftDesktopToolsTest"),
                    ..Default::default()
                };
                RegisterClassW(&class);
                let title: Vec<u16> = TITLE.encode_utf16().chain(Some(0)).collect();
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BotloftDesktopToolsTest"),
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
                .expect("window");
                let child = |class: PCWSTR, text: PCWSTR, style: u32, top: i32, id: usize| {
                    CreateWindowExW(
                        WINDOW_EX_STYLE::default(),
                        class,
                        text,
                        WS_CHILD | WS_VISIBLE | WINDOW_STYLE(style),
                        10,
                        top,
                        160,
                        24,
                        Some(hwnd),
                        Some(HMENU(id as *mut std::ffi::c_void)),
                        None,
                        None,
                    )
                    .expect("control");
                };
                child(w!("BUTTON"), w!("Save"), BS_PUSHBUTTON as u32, 10, 1);
                child(w!("EDIT"), w!("Ana Lima"), WS_BORDER.0, 40, 2);
                child(
                    w!("EDIT"),
                    w!("hunter2"),
                    WS_BORDER.0 | ES_PASSWORD as u32,
                    70,
                    3,
                );
                sent.send(hwnd.0 as usize as u64).expect("send");
                let mut message = MSG::default();
                while GetMessageW(&mut message, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        });
        let id = opened.recv_timeout(Duration::from_secs(5)).expect("opened");
        std::thread::sleep(Duration::from_millis(300));
        Self {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        let hwnd = HWND(self.id as usize as *mut std::ffi::c_void);
        // SAFETY: asks the window's own thread to close it.
        let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Setup {
    pub t: TestDaemon,
    pub app: Client,
    pub bot: Value,
    pub mcp: Mcp,
}

pub async fn setup() -> Setup {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Clinic" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    Setup { t, app, bot, mcp }
}

pub fn reading(result: &Result<String, String>) -> &str {
    match result {
        Ok(text) | Err(text) => text,
    }
}

/// The ref of the line that has `what` in a reading.
pub fn ref_in(reading: &str, what: &str) -> String {
    let line = reading
        .lines()
        .find(|line| line.contains(what))
        .unwrap_or_else(|| panic!("{what} is not in the reading:\n{reading}"));
    let start = line.find('[').expect("a ref") + 1;
    let end = line.find(']').expect("a ref");
    line[start..end].to_owned()
}
