//! The phone (spec 28): connecting one with the QR code, and approving and
//! answering from it through the relay of the real account server.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use botloft_core::protocol::{ApprovalStatus, FromPhone, QuestionStatus, ToPhone};
use common::bots::ready_bot;
use common::cloud_server::{cloud_server_pushing, sign_in};
use common::mcp::Mcp;
use common::mobile::{connect_phone, join, setup, status_where};
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

/// A lead in a turn, ready to ask for permission.
async fn lead(
    t: &TestDaemon,
    app: &mut Client,
) -> (
    botloftd::runtime::fake::FakeProcess,
    std::net::SocketAddr,
    String,
) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, mcp) = ready_bot(t, app, &crew, "Lead").await;
    app.call("messages.send", json!({ "botId": bot["id"], "body": "Go" }))
        .await
        .expect("send");
    let lines = process.wait_lines(1).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    (process, mcp.addr, mcp.token.clone())
}

/// Claude Code asks for permission; the decision comes out of the handle.
async fn ask(
    process: &botloftd::runtime::fake::FakeProcess,
    at: &(std::net::SocketAddr, String),
    id: &str,
    tool: &str,
    input: &Value,
) -> tokio::task::JoinHandle<Value> {
    process
        .emit(stream::tool_use(id, tool, input.clone()))
        .await;
    let mut mcp = Mcp::new(at.0, at.1.clone());
    let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

#[tokio::test]
async fn a_phone_is_connected_with_the_code_and_answers_a_question_once() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;

    // The keys are on this computer, in the owner's secrets, and nowhere else.
    let kept = std::fs::read_to_string(t.paths.secrets().join("mobile.json")).expect("keys kept");
    assert!(kept.contains(&phone.id));
    let status = app
        .call("mobile.status", Value::Null)
        .await
        .expect("status");
    assert_eq!(status["relay"], "connected");
    assert_eq!(status["phones"][0]["name"], "Celular da Ana");
    assert!(status.get("pending").is_none());

    phone.send(&FromPhone::Sync).await;
    match phone.next().await {
        ToPhone::Snapshot {
            approvals,
            questions,
        } => assert!(approvals.is_empty() && questions.is_empty()),
        other => panic!("{other:?}"),
    }

    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (_, process, mut mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    mcp.tool(
        "ask_owner",
        json!({ "question": "Which client first?", "options": ["Acme", "Globex"] }),
    )
    .await
    .expect("asked");
    let ToPhone::QuestionOpen { card } = phone.next().await else {
        panic!("the question should reach the phone");
    };
    assert_eq!(
        (
            card.text.as_str(),
            card.bot.name.as_str(),
            card.crew.as_str()
        ),
        ("Which client first?", "Scout", "Ops")
    );
    assert_eq!(card.options, ["Acme", "Globex"]);

    let sent = phone
        .send(&FromPhone::QuestionAnswer {
            question_id: card.question_id.clone(),
            answer: "Acme".into(),
        })
        .await;
    let ToPhone::QuestionClosed { status, .. } = phone.next().await else {
        panic!("the card should close");
    };
    assert_eq!(status, QuestionStatus::Answered);
    let lines = process.wait_lines(1).await;
    assert!(common::bots::text_of(&lines[0]).contains("Acme"));
    // (The app heard the question open first.)
    while app.notification("question.changed").await["status"] != "answered" {}

    // The same sealed message again does nothing: not a second answer, not a reply.
    phone.send_raw(sent).await;
    phone.quiet().await;
    assert_eq!(process.input_lines().len(), 1);

    // A message changed on the way does not open and does not count.
    let mut tampered = phone.seal(&FromPhone::Sync);
    let body = tampered["body"]
        .as_str()
        .expect("body")
        .replace("\"ct\":\"", "\"ct\":\"AAAA");
    tampered["body"] = Value::String(body);
    phone.send_raw(tampered).await;
    phone.send(&FromPhone::Sync).await;
    assert!(matches!(phone.next().await, ToPhone::Snapshot { .. }));
}

#[tokio::test]
async fn the_phone_allows_what_it_can_read_whole_and_only_denies_the_rest() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let (process, addr, token) = lead(&t, &mut app).await;
    let at = (addr, token);

    // A plain command: shown whole, allowed from the phone.
    let input = json!({ "command": "git status", "description": "Shows what changed" });
    let decision = ask(&process, &at, "toolu_1", "Bash", &input).await;
    let ToPhone::ApprovalOpen { card } = phone.next().await else {
        panic!("the request should reach the phone");
    };
    assert_eq!(
        (
            card.tool_name.as_str(),
            card.text.as_str(),
            card.bot.name.as_str()
        ),
        ("Bash", "git status", "Lead")
    );
    assert_eq!(card.explanation.as_deref(), Some("Shows what changed"));
    assert!(!card.cut && !card.at_computer);
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: true,
            note: None,
        })
        .await;
    assert_eq!(decision.await.expect("task")["behavior"], "allow");
    assert!(matches!(
        phone.next().await,
        ToPhone::ApprovalClosed {
            status: ApprovalStatus::Allowed,
            ..
        }
    ));

    // A tool that belongs to the computer: the phone cannot allow it, and is shown so.
    let input = json!({ "title": "Fix it" });
    let decision = ask(
        &process,
        &at,
        "toolu_2",
        "mcp__github__create_issue",
        &input,
    )
    .await;
    let ToPhone::ApprovalOpen { card } = phone.next().await else {
        panic!("open")
    };
    assert!(card.at_computer);
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: true,
            note: None,
        })
        .await;
    let ToPhone::ApprovalOpen { card: again } = phone.next().await else {
        panic!("the card again")
    };
    assert_eq!(again.approval_id, card.approval_id);
    assert!(!decision.is_finished(), "still waiting for the owner");
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: false,
            note: Some("not now".into()),
        })
        .await;
    assert_eq!(decision.await.expect("task")["behavior"], "deny");
    assert!(matches!(
        phone.next().await,
        ToPhone::ApprovalClosed {
            status: ApprovalStatus::Denied,
            ..
        }
    ));

    // A command too long for a phone is cut and marked: deny only.
    let input = json!({ "command": format!("echo {}", "x".repeat(9000)) });
    let decision = ask(&process, &at, "toolu_3", "Bash", &input).await;
    let ToPhone::ApprovalOpen { card } = phone.next().await else {
        panic!("open")
    };
    assert!(card.cut && card.text.len() <= 8 * 1024);
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: true,
            note: None,
        })
        .await;
    assert!(matches!(phone.next().await, ToPhone::ApprovalOpen { .. }));
    assert!(!decision.is_finished());
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: false,
            note: None,
        })
        .await;
    assert_eq!(decision.await.expect("task")["behavior"], "deny");
    assert!(matches!(phone.next().await, ToPhone::ApprovalClosed { .. }));
}

#[tokio::test]
async fn an_answer_at_the_computer_closes_the_card_and_a_late_answer_changes_nothing() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let (process, addr, token) = lead(&t, &mut app).await;
    let at = (addr, token);

    let decision = ask(
        &process,
        &at,
        "toolu_1",
        "Bash",
        &json!({ "command": "ls" }),
    )
    .await;
    let ToPhone::ApprovalOpen { card } = phone.next().await else {
        panic!("open")
    };

    // A phone that opens later is told what waits.
    phone.send(&FromPhone::Sync).await;
    let ToPhone::Snapshot { approvals, .. } = phone.next().await else {
        panic!("snapshot")
    };
    assert_eq!(approvals.len(), 1);
    assert_eq!(approvals[0].approval_id, card.approval_id);

    app.call(
        "approvals.answer",
        json!({ "approvalId": card.approval_id, "allow": false }),
    )
    .await
    .expect("answer");
    assert_eq!(decision.await.expect("task")["behavior"], "deny");
    assert!(matches!(
        phone.next().await,
        ToPhone::ApprovalClosed {
            status: ApprovalStatus::Denied,
            ..
        }
    ));

    // The phone allows it anyway, late: it is told how it ended, nothing is allowed.
    phone
        .send(&FromPhone::ApprovalAnswer {
            approval_id: card.approval_id.clone(),
            allow: true,
            note: None,
        })
        .await;
    assert!(matches!(
        phone.next().await,
        ToPhone::ApprovalClosed {
            status: ApprovalStatus::Denied,
            ..
        }
    ));
}

#[tokio::test]
async fn a_wrong_proof_or_a_refusal_ends_the_code_without_a_phone() {
    let (cloud, _t, mut app) = setup().await;
    let started = app
        .call("mobile.pair_start", Value::Null)
        .await
        .expect("start");
    let qr = started["url"].as_str().expect("url");
    let _ = join(&cloud.url, qr, "Intruder", true).await;
    status_where(&mut app, |s| s.get("pending").is_none()).await;
    let status = app
        .call("mobile.status", Value::Null)
        .await
        .expect("status");
    assert_eq!(status["phones"], json!([]));

    // The owner sees a phone and says no.
    let started = app
        .call("mobile.pair_start", Value::Null)
        .await
        .expect("start");
    let joined = join(
        &cloud.url,
        started["url"].as_str().expect("url"),
        "Celular",
        false,
    )
    .await;
    app.notification("mobile.pair_request").await;
    app.call(
        "mobile.pair_confirm",
        json!({ "pairId": started["pairId"], "accept": false }),
    )
    .await
    .expect("refuse");
    assert!(joined.collect().await.is_none(), "the phone gets no token");
    status_where(&mut app, |s| s.get("pending").is_none()).await;
    let nothing = app
        .call(
            "mobile.pair_confirm",
            json!({ "pairId": started["pairId"], "accept": true }),
        )
        .await
        .expect_err("nothing to confirm");
    assert_eq!(nothing.code, -32003);
}

#[tokio::test]
async fn revoking_a_phone_cuts_it_off_and_leaving_the_account_takes_every_phone() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let gone = app
        .call("mobile.revoke", json!({ "phoneId": phone.id }))
        .await
        .expect("revoke");
    assert_eq!(gone["phones"], json!([]));
    phone.closed().await;
    let me = reqwest::Client::new()
        .get(format!("{}/v1/me", cloud.url))
        .bearer_auth(&phone.token)
        .send()
        .await
        .expect("me");
    assert_eq!(me.status(), 401, "the server dropped the phone's token");
    status_where(&mut app, |s| s["relay"] == "off").await;
    let unknown = app
        .call("mobile.revoke", json!({ "phoneId": phone.id }))
        .await
        .expect_err("already gone");
    assert_eq!(unknown.code, -32002);

    // Another phone, then the computer leaves the account.
    let mut second = connect_phone(&cloud, &mut app).await;
    app.call("cloud.signout", Value::Null)
        .await
        .expect("signout");
    second.closed().await;
    assert!(!t.paths.secrets().join("mobile.json").exists());
    let status = app
        .call("mobile.status", Value::Null)
        .await
        .expect("status");
    assert_eq!(status["phones"], json!([]));
}

#[tokio::test]
async fn the_relay_comes_back_when_the_server_cuts_it_and_nothing_is_lost_for_good() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let credentials: Value = serde_json::from_slice(
        &std::fs::read(t.paths.secrets().join("cloud.json")).expect("credentials"),
    )
    .expect("json");
    cloud
        .hub
        .kick(credentials["device"].as_str().expect("device"));

    // It reconnects by itself, and the phone is online again.
    status_where(&mut app, |s| {
        s["relay"] == "connected" && s["phones"][0]["online"] == true
    })
    .await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (_, _process, mut mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    mcp.tool("ask_owner", json!({ "question": "After the drop?" }))
        .await
        .expect("asked");
    // The status can still say "connected" for a moment after the cut, so the
    // question may open while the relay is down. A phone that is told its
    // computer is back asks again (as the page does), and gets the retrato.
    phone.send(&FromPhone::Sync).await;
    let card = loop {
        match phone.next().await {
            ToPhone::QuestionOpen { card } => break card,
            ToPhone::Snapshot { questions, .. } if !questions.is_empty() => {
                break questions[0].clone();
            }
            _ => {}
        }
    };
    assert_eq!(card.text, "After the drop?");
}

/// What a push service was asked: the authorization and the size of the body.
type Asked = Arc<Mutex<Vec<(String, usize)>>>;

#[tokio::test]
async fn a_question_that_opens_makes_the_server_push_the_phone_with_nothing_in_it() {
    // A push service that counts what it is asked, and what came with it.
    let asked: Asked = Arc::default();
    let service = axum::Router::new()
        .route(
            "/{*any}",
            axum::routing::post(
                |axum::extract::State(asked): axum::extract::State<Asked>,
                 headers: axum::http::HeaderMap,
                 body: axum::body::Bytes| async move {
                    let auth = headers
                        .get("authorization")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or_default()
                        .to_owned();
                    asked.lock().expect("asked").push((auth, body.len()));
                    axum::http::StatusCode::CREATED
                },
            ),
        )
        .with_state(Arc::clone(&asked));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, service).await;
    });

    let (private, _) = botloft_cloud::new_key().expect("key");
    let pusher = botloft_cloud::Pusher::new(
        &private,
        "mailto:ops@example.org",
        &["127.0.0.1".to_owned()],
        Duration::from_millis(50),
    )
    .expect("pusher");
    let cloud = cloud_server_pushing(pusher).await;
    let t = TestDaemon::start_supervised_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    sign_in(&cloud, &mut app).await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let subscribed = reqwest::Client::new()
        .post(format!("{}/v1/push/subscribe", cloud.url))
        .bearer_auth(&phone.token)
        .json(&json!({ "endpoint": format!("http://{address}/push/abc") }))
        .send()
        .await
        .expect("subscribe");
    assert_eq!(subscribed.status(), 204);

    // A bot asks: the owner's phone is told, and the push holds nothing of it.
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (_, _process, mut mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    mcp.tool("ask_owner", json!({ "question": "A secret question?" }))
        .await
        .expect("asked");
    for _ in 0..100 {
        if !asked.lock().expect("asked").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let pushes = asked.lock().expect("asked").clone();
    assert_eq!(pushes.len(), 1);
    assert!(pushes[0].0.starts_with("vapid t="), "{}", pushes[0].0);
    assert_eq!(pushes[0].1, 0, "an empty push");
    assert!(!pushes[0].0.contains("secret"));
    // The question itself reaches the phone, sealed, through the relay.
    let ToPhone::QuestionOpen { card } = phone.next().await else {
        panic!("the question should reach the phone");
    };
    assert_eq!(card.text, "A secret question?");
}
