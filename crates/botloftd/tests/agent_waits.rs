//! Waits without the model (spec 20.14): `wait_for` sets a wait on a file,
//! a program or a page; Botloft looks by itself and wakes the bot once,
//! when it happened or when the time ran out.

mod common;

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use botloftd::routines::waits::{self, PageText, WaitProbe};
use common::bots::{ready_bot, text_of};
use common::mcp::Mcp;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

/// Real files; programs and a page as the test sets them.
#[derive(Default)]
struct Fake {
    programs: Mutex<Option<Vec<String>>>,
    page: Mutex<String>,
    reads: AtomicUsize,
}

impl Fake {
    fn set_programs(&self, names: &[&str]) {
        *self.programs.lock().expect("programs") =
            Some(names.iter().map(|name| (*name).to_owned()).collect());
    }
}

impl WaitProbe for Fake {
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn programs(&self) -> Option<Vec<String>> {
        self.programs.lock().expect("programs").clone()
    }

    fn page(&self, _url: &str) -> PageText<'_> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let text = self.page.lock().expect("page").clone();
        Box::pin(async move { Some(text) })
    }
}

struct One {
    t: TestDaemon,
    app: Client,
    bot: Value,
    process: botloftd::runtime::fake::FakeProcess,
    mcp: Mcp,
    fake: Arc<Fake>,
}

async fn one_bot() -> One {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    let fake = Arc::new(Fake::default());
    t.daemon
        .routines
        .set_probe(Arc::clone(&fake) as Arc<dyn WaitProbe>);
    One {
        t,
        app,
        bot,
        process,
        mcp,
        fake,
    }
}

async fn runs(app: &mut Client, routine: &Value) -> Vec<Value> {
    app.call("routines.runs", json!({ "routineId": routine }))
        .await
        .expect("runs")
        .as_array()
        .expect("list")
        .clone()
}

async fn waits_of(app: &mut Client, bot: &Value) -> Vec<Value> {
    app.call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines")
        .as_array()
        .expect("list")
        .clone()
}

#[tokio::test]
async fn a_file_wait_wakes_the_bot_once_when_the_file_appears() {
    let One {
        t,
        mut app,
        bot,
        process,
        mut mcp,
        ..
    } = one_bot().await;
    let set = mcp
        .tool(
            "wait_for",
            json!({ "name": "Wait for the report", "what": "file", "path": "out/report.pdf",
                    "hours": 2, "then": "Review the report." }),
        )
        .await
        .expect("set");
    assert_eq!(set["created"], true);
    let waiting = set["waiting_for"].as_str().expect("waiting_for");
    let path = waiting
        .strip_prefix("the file ")
        .and_then(|rest| rest.strip_suffix(" to appear"))
        .expect("the path")
        .to_owned();
    assert!(path.ends_with("report.pdf"), "{path}");
    assert!(Path::new(&path).is_absolute(), "{path}");

    let listed = waits_of(&mut app, &bot).await;
    assert_eq!(listed.len(), 1);
    let wait = &listed[0];
    assert_eq!(wait["schedule"]["kind"], "wait");
    assert_eq!(
        wait["schedule"]["on"],
        json!({ "what": "file", "path": path })
    );
    assert_eq!(wait["nextRunAt"], Value::Null);
    assert_eq!(wait["prompt"], "Review the report.");

    waits::check(&t.daemon).await;
    assert!(runs(&mut app, &wait["id"]).await.is_empty());

    let file = Path::new(&path);
    std::fs::create_dir_all(file.parent().expect("folder")).expect("folder");
    std::fs::write(file, b"%PDF").expect("file");
    waits::check(&t.daemon).await;
    let done = runs(&mut app, &wait["id"]).await;
    assert_eq!(done.len(), 1);
    assert_eq!(done[0]["waited"], "happened");

    let lines = process.wait_lines(1).await;
    let text = text_of(&lines[0]);
    let first = format!("[botloft] wait \"Wait for the report\" · the file {path} appeared\n");
    assert!(text.starts_with(&first), "{text}");
    assert!(text.ends_with("Review the report."), "{text}");

    // It went once it ran.
    assert!(waits_of(&mut app, &bot).await.is_empty());
    waits::check(&t.daemon).await;
    assert_eq!(runs(&mut app, &wait["id"]).await.len(), 1);
}

#[tokio::test]
async fn a_program_ends_and_a_page_runs_out() {
    let One {
        t,
        mut app,
        bot,
        process,
        mut mcp,
        fake,
    } = one_bot().await;
    fake.set_programs(&["explorer.exe", "ffmpeg.exe"]);
    *fake.page.lock().expect("page") = "<p>Pedido em preparo</p>".to_owned();

    let err = mcp
        .tool(
            "wait_for",
            json!({ "name": "Render", "what": "program", "program": "blender",
                    "hours": 1, "then": "Check the render." }),
        )
        .await
        .expect_err("not running");
    assert!(err.contains("no program called blender"), "{err}");
    let program = mcp
        .tool(
            "wait_for",
            json!({ "name": "Video", "what": "program", "program": "FFmpeg",
                    "hours": 1, "then": "Upload the video." }),
        )
        .await
        .expect("program");
    assert_eq!(program["waiting_for"], "FFmpeg to end");
    let page = mcp
        .tool(
            "wait_for",
            json!({ "name": "Order", "what": "page", "url": "https://shop.test/order",
                    "text": "Enviado", "hours": 1, "then": "Tell the owner." }),
        )
        .await
        .expect("page");

    waits::check(&t.daemon).await;
    waits::check(&t.daemon).await;
    assert_eq!(
        fake.reads.load(Ordering::SeqCst),
        1,
        "a page at most once a minute"
    );
    assert!(runs(&mut app, &program["routine_id"]).await.is_empty());

    fake.set_programs(&["explorer.exe"]);
    waits::check(&t.daemon).await;
    let ended = runs(&mut app, &program["routine_id"]).await;
    assert_eq!(ended[0]["waited"], "happened");
    let lines = process.wait_lines(1).await;
    let text = text_of(&lines[0]);
    assert!(
        text.starts_with(
            "[botloft] wait \"Video\" · no program called FFmpeg is running any more\n"
        ),
        "{text}"
    );
    process.emit(stream::init("session-1")).await;
    process.emit(stream::replay(&lines[0])).await;
    process.emit(stream::result(false)).await;

    t.clock.advance(Duration::from_secs(61 * 60));
    waits::check(&t.daemon).await;
    assert_eq!(fake.reads.load(Ordering::SeqCst), 2);
    let late = runs(&mut app, &page["routine_id"]).await;
    assert_eq!(late[0]["waited"], "timed_out");
    let lines = process.wait_lines(2).await;
    let text = text_of(&lines[1]);
    assert!(
        text.starts_with("[botloft] wait \"Order\" · ran out at "),
        "{text}"
    );
    assert!(
        text.contains("the page https://shop.test/order does not show \"Enviado\""),
        "{text}"
    );
    assert!(waits_of(&mut app, &bot).await.is_empty());
}

#[tokio::test]
async fn only_an_agent_sets_a_wait_inside_its_folders_and_few_at_once() {
    // `t` holds the daemon's folder: keep it.
    let One {
        t: _t,
        mut app,
        bot,
        mut mcp,
        ..
    } = one_bot().await;
    let schedule = json!({ "kind": "wait", "on": { "what": "program", "name": "x" }, "until": 1 });
    let refused = app
        .call(
            "routines.create",
            json!({ "botId": bot["id"], "name": "Mine", "prompt": "Go.",
                    "schedule": schedule, "timezone": "UTC" }),
        )
        .await
        .expect_err("only agents");
    assert_eq!(refused.reason, "wait_by_agent");

    let outside = std::env::temp_dir().join("botloft-outside.txt");
    let tries = [
        (
            json!({ "path": outside.display().to_string() }),
            "outside your folder",
        ),
        (json!({ "path": "../elsewhere.txt" }), "must not go up"),
        (json!({ "path": "CLAUDE.md" }), "already exists"),
        (
            json!({ "path": "a.txt", "url": "https://x.test" }),
            "takes path, and nothing else",
        ),
        (json!({ "path": "a.txt", "hours": 0 }), "between 1 and 168"),
        (
            json!({ "path": "a.txt", "hours": 169 }),
            "between 1 and 168",
        ),
        (
            json!({ "path": "a.txt", "what": "mood" }),
            "file, program or page",
        ),
    ];
    for (extra, expected) in tries {
        let mut args = json!({ "name": "Wait", "what": "file", "hours": 1, "then": "Go on." });
        for (key, value) in extra.as_object().expect("extra") {
            args[key] = value.clone();
        }
        let err = mcp.tool("wait_for", args).await.expect_err(expected);
        assert!(err.contains(expected), "{expected}: {err}");
    }

    for n in 0..5 {
        mcp.tool(
            "wait_for",
            json!({ "name": format!("Wait {n}"), "what": "file", "path": format!("w{n}.txt"),
                    "hours": 1, "then": "Go on." }),
        )
        .await
        .expect("a wait");
    }
    let err = mcp
        .tool(
            "wait_for",
            json!({ "name": "One more", "what": "file", "path": "w5.txt",
                    "hours": 1, "then": "Go on." }),
        )
        .await
        .expect_err("too many");
    assert!(err.contains("5 waits open"), "{err}");

    // The owner may rename one, but not change when it runs.
    let wait = waits_of(&mut app, &bot).await[0]["id"].clone();
    app.call(
        "routines.update",
        json!({ "routineId": wait, "name": "Renamed" }),
    )
    .await
    .expect("renamed");
    let refused = app
        .call(
            "routines.update",
            json!({ "routineId": wait,
                    "schedule": { "kind": "weekly", "days": [1], "time": "09:00" } }),
        )
        .await
        .expect_err("its time stays");
    assert_eq!(refused.reason, "wait_by_agent");
}
