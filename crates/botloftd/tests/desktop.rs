//! A bot on the owner's desktop with real windows (spec 24): a window of
//! the test's own, with a button, a field, a password field and a check
//! box, seen once the owner lets the bot see its app. Windows only.
#![cfg(windows)]

mod common;

use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use common::Client;
use common::TestDaemon;
use common::bots::ready_bot;
use common::browsing::{call, pending_approval};
use common::mcp::Mcp;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    BS_PUSHBUTTON, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_PASSWORD,
    GetMessageW, HMENU, MSG, PostMessageW, PostQuitMessage, RegisterClassW, TranslateMessage,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_BORDER, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

const TITLE: &str = "Botloft desktop: the clinic";

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

/// A window of the test's own, on a thread with its message loop.
struct TestWindow {
    id: u64,
    thread: Option<JoinHandle<()>>,
}

impl TestWindow {
    fn open() -> Self {
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
                let child = |class: PCWSTR, text: PCWSTR, style: u32, top: i32| {
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
                        Some(HMENU(std::ptr::null_mut())),
                        None,
                        None,
                    )
                    .expect("control");
                };
                child(w!("BUTTON"), w!("Save"), BS_PUSHBUTTON as u32, 10);
                child(w!("EDIT"), w!("Ana Lima"), WS_BORDER.0, 40);
                child(
                    w!("EDIT"),
                    w!("hunter2"),
                    WS_BORDER.0 | ES_PASSWORD as u32,
                    70,
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

struct Setup {
    t: TestDaemon,
    app: Client,
    bot: Value,
    mcp: Mcp,
}

async fn setup() -> Setup {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Clinic" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    Setup { t, app, bot, mcp }
}

fn reading(result: &Result<String, String>) -> &str {
    match result {
        Ok(text) | Err(text) => text,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_sees_an_app_once_the_owner_lets_it_and_never_a_password() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let look = json!({ "window": window.id, "why": "To read the patient's name" });

    // Before any grant, the window is there without its title.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(
        listed.contains(&format!("window {}:", window.id)),
        "{listed}"
    );
    assert!(!listed.contains(TITLE), "{listed}");

    // Reading it asks the owner, with the app and the bot's why.
    let reading_it = call(&s.mcp, "desktop_look", look.clone());
    let asked = pending_approval(&mut s.app).await;
    assert_eq!(asked["toolName"], "mcp__botloft__desktop");
    assert!(
        asked["input"]
            .as_str()
            .is_some_and(|input| input.contains("To read the patient's name")),
        "{asked}"
    );
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("answer");
    let text = reading_it.await.expect("task").expect("reading");
    assert!(text.contains(TITLE), "{text}");
    assert!(text.contains(r#"button "Save""#), "{text}");
    assert!(text.contains(r#"= "Ana Lima""#), "{text}");
    assert!(!text.contains("hunter2"), "{text}");

    // The grant is kept and the apps hear of it.
    let changed = s.app.notification("bot.desktop").await;
    assert_eq!(changed["grants"][0]["level"], "see");
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    assert_eq!(grants.as_array().map(Vec::len), Some(1));

    // Now the title shows, and so does a picture, without asking again.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(listed.contains(TITLE), "{listed}");
    let picture = s.mcp.tool_result("desktop_screenshot", look.clone()).await;
    assert_eq!(picture["content"][0]["type"], "image");
    assert_eq!(picture["content"][0]["mimeType"], "image/jpeg");

    // Taken away, it asks again; refused, the bot reads why.
    let revoke = json!({ "grantId": grants[0]["id"] });
    s.app.call("desktop.revoke", revoke).await.expect("revoke");
    let reading_it = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let deny = json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Not now" });
    s.app.call("approvals.answer", deny).await.expect("answer");
    let refused = reading_it.await.expect("task");
    assert!(refused.is_err());
    assert!(
        reading(&refused).contains("Not now"),
        "{}",
        reading(&refused)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_seen_while_the_owner_is_away() {
    let s = setup().await;
    s.t.daemon
        .desktop
        .set_owner_idle(std::sync::Arc::new(|| Some(Duration::from_secs(3600))));
    let mut mcp = s.mcp;
    let away = mcp.tool_text("desktop_windows", json!({})).await;
    assert!(away.is_err());
    assert!(
        reading(&away).contains("has not used the computer"),
        "{}",
        reading(&away)
    );
}
