//! By hand: a real browser as the phone (spec 28, CE5). The account server
//! serves the built page, a daemon with fake bots asks for permission and asks
//! a question, and the test waits for a person to connect the browser and
//! answer. It writes the address to open in `.dev/live/pair.txt`.
//!
//! ```text
//! pnpm --dir app build:phone
//! cargo test -p botloftd --test mobile_live -- --ignored --nocapture
//! ```
//!
//! Open the address in the browser, press Connect, compare the six digits it
//! shows with the ones in `.dev/live/code.txt` (the test accepts by itself),
//! then Allow or Deny the requests and answer the question.

mod common;

use std::time::Duration;

use common::bots::ready_bot;
use common::cloud_server::{cloud_server_on, sign_in};
use common::mcp::Mcp;
use common::{TestDaemon, stream};
use serde_json::{Value, json};
use tokio::net::TcpListener;

const PORT: u16 = 8799;

#[tokio::test]
#[ignore = "needs a person with a browser"]
async fn a_real_browser_connects_and_answers() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/dist-phone");
    assert!(
        dist.join("index.html").exists(),
        "build the page first: pnpm --dir app build:phone"
    );
    let live = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.dev/live");
    std::fs::create_dir_all(&live).expect("folder");

    let listener = TcpListener::bind(("127.0.0.1", PORT))
        .await
        .expect("the port is free");
    let cloud = cloud_server_on(listener, Some(dist)).await;
    let t = TestDaemon::start_supervised_with_cloud_waiting(&cloud.url, Duration::from_secs(1200))
        .await;
    let mut app = t.session().await;
    sign_in(&cloud, &mut app).await;

    let started = app
        .call("mobile.pair_start", Value::Null)
        .await
        .expect("start");
    let url = started["url"].as_str().expect("url");
    std::fs::write(live.join("pair.txt"), url).expect("write");
    println!("OPEN THIS IN THE BROWSER:\n{url}");

    // (Not a notification: the test's client gives up on those after 20 s.)
    let mut request = Value::Null;
    for _ in 0..6000 {
        let status = app
            .call("mobile.status", Value::Null)
            .await
            .expect("status");
        if status["pending"]["joined"].is_object() {
            request = status["pending"]["joined"].clone();
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(request.is_object(), "a phone joined in ten minutes");
    println!("a phone joined: {} {}", request["name"], request["code"]);
    std::fs::write(
        live.join("code.txt"),
        request["code"].as_str().expect("code"),
    )
    .expect("write");
    app.call(
        "mobile.pair_confirm",
        json!({ "pairId": started["pairId"], "accept": true }),
    )
    .await
    .expect("confirm");
    for _ in 0..200 {
        let status = app
            .call("mobile.status", Value::Null)
            .await
            .expect("status");
        if status["phones"][0]["online"] == true {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    println!("the phone is connected");

    // A lead in a turn, and four things for the owner.
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, mut mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    app.call("messages.send", json!({ "botId": bot["id"], "body": "Go" }))
        .await
        .expect("send");
    let lines = process.wait_lines(1).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");

    let asks = [
        (
            "toolu_1",
            "Bash",
            json!({ "command": "git status", "description": "Shows what changed in the project" }),
        ),
        (
            "toolu_2",
            "Bash",
            json!({ "command": format!("echo {}", "x".repeat(9000)) }),
        ),
        (
            "toolu_3",
            "mcp__github__create_issue",
            json!({ "title": "Fix the login" }),
        ),
    ];
    let mut decisions = Vec::new();
    for (id, tool, input) in asks {
        process
            .emit(stream::tool_use(id, tool, input.clone()))
            .await;
        let mut client = Mcp::new(t.addr, token.clone());
        let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
        decisions.push((
            tool,
            tokio::spawn(async move {
                client
                    .tool("permission_prompt", args)
                    .await
                    .expect("decision")
            }),
        ));
    }
    mcp.tool(
        "ask_owner",
        json!({ "question": "Which **client** do we call first?", "options": ["Acme", "Globex"] }),
    )
    .await
    .expect("asked");
    println!("waiting for the answers…");

    for (tool, decision) in decisions {
        let outcome = tokio::time::timeout(Duration::from_secs(600), decision)
            .await
            .expect("answered in ten minutes")
            .expect("task");
        println!("{tool}: {}", outcome["behavior"]);
    }
    for _ in 0..6000 {
        let open = app.call("questions.list", json!({})).await.expect("list");
        if open.as_array().is_some_and(Vec::is_empty) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let answered = app
        .call("questions.list", json!({ "status": "answered" }))
        .await
        .expect("list");
    println!("question: {answered}");
}
