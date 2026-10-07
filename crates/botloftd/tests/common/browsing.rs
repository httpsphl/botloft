//! The browser tests' shared setup (spec 21): a bot in Manual mode, a local
//! site, and helpers to call tools in the background and answer requests.

use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinHandle;

use super::bots::ready_bot;
use super::mcp::Mcp;
use super::{Client, TestDaemon, site};

pub struct Browsing {
    pub t: TestDaemon,
    pub app: Client,
    pub bot: Value,
    pub mcp: Mcp,
    pub site: String,
    /// Held until the test ends, after the daemon has closed its browser.
    _slot: OwnedSemaphorePermit,
}

/// Real browsers a test binary has open at once. A dozen cold Chromium
/// profiles starting together on a small CI runner could pass the daemon's
/// 30 s start limit, so the tests wait their turn instead.
const BROWSERS_AT_ONCE: usize = 3;

async fn browser_slot() -> OwnedSemaphorePermit {
    static SLOTS: OnceLock<std::sync::Arc<Semaphore>> = OnceLock::new();
    let slots = SLOTS.get_or_init(|| std::sync::Arc::new(Semaphore::new(BROWSERS_AT_ONCE)));
    slots.clone().acquire_owned().await.expect("open semaphore")
}

/// A bot in Manual mode and a local site, or `None` without Edge.
pub async fn setup() -> Option<Browsing> {
    if botloftd::browser::find_program("").is_none() {
        eprintln!("Microsoft Edge is not installed; skipping");
        return None;
    }
    let slot = browser_slot().await;
    // A real browser can be slow to ask on a busy CI runner.
    let t = TestDaemon::start_supervised_patient().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Web" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    let site = format!("http://{}", site::serve().await);
    Some(Browsing {
        t,
        app,
        bot,
        mcp,
        site,
        _slot: slot,
    })
}

pub fn call(mcp: &Mcp, name: &str, arguments: Value) -> JoinHandle<Result<String, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let name = name.to_owned();
    tokio::spawn(async move { mcp.tool_text(&name, arguments).await })
}

/// Waits for the bot to ask about a site and answers.
pub async fn answer_site(app: &mut Client, allow: bool, note: Option<&str>) -> Value {
    let asked = pending_approval(app).await;
    assert_eq!(asked["toolName"], "mcp__botloft__browser");
    let mut answer = json!({ "approvalId": asked["approvalId"], "allow": allow });
    if let Some(note) = note {
        answer["note"] = json!(note);
    }
    app.call("approvals.answer", answer).await.expect("answer");
    asked
}

/// The next request that shows up in the chat.
pub async fn pending_approval(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            return body.clone();
        }
    }
}

/// The ref of the first control on the page whose line has `what`.
pub fn ref_of(page: &str, what: &str) -> String {
    let start = page
        .find(what)
        .unwrap_or_else(|| panic!("{what} is not on the page:\n{page}"));
    let open = page[..start].rfind('[').expect("a control");
    page[open + 1..]
        .split_whitespace()
        .next()
        .expect("ref")
        .to_owned()
}

/// One of the owner's events, as the app sends it.
pub async fn send(app: &mut Client, bot: &Value, input: Value) {
    app.call(
        "browser.input",
        json!({ "botId": bot["id"], "input": input }),
    )
    .await
    .expect("input");
}

/// A left click at a point of the page.
pub async fn click(app: &mut Client, bot: &Value, x: f64, y: f64) {
    for (action, buttons) in [("move", 0), ("down", 1), ("up", 0)] {
        let input = json!({
            "kind": "mouse", "action": action, "x": x, "y": y, "button": "left",
            "buttons": buttons, "clicks": 1, "modifiers": 0,
        });
        send(app, bot, input).await;
    }
}

/// Presses the keys of `text`, one by one, as the app sends them.
pub async fn press(app: &mut Client, bot: &Value, text: &str, modifiers: u32) {
    for key in text.chars() {
        let code = format!("Key{}", key.to_ascii_uppercase());
        let input =
            json!({ "kind": "key", "key": key.to_string(), "code": code, "modifiers": modifiers });
        send(app, bot, input).await;
    }
}
