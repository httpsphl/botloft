//! The phone's conversations (spec 28.12): the list, the history, writing to
//! a bot and following a reply as it is written.

mod common;

use std::time::Duration;

use botloft_core::protocol::{
    BotState, ChatLine, FromPhone, ITEM_TEXT_MAX, PhoneItem, SEALED_MAX, ToPhone,
};
use common::bots::ready_bot;
use common::mobile::{Phone, connect_phone, setup};
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

fn bot_id(bot: &Value) -> botloft_core::ids::BotId {
    bot["id"].as_str().expect("id").parse().expect("bot id")
}

/// A crew with one running bot, and a phone.
async fn start() -> (TestDaemon, Client, Phone, Value) {
    let (cloud, t, mut app) = setup().await;
    let phone = connect_phone(&cloud, &mut app).await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Lead").await;
    drop(process);
    (t, app, phone, bot)
}

/// Asks for a page and puts its parts together.
async fn history(phone: &mut Phone, bot: &Value, req: u32) -> (Vec<PhoneItem>, bool) {
    phone
        .send(&FromPhone::History {
            req,
            bot_id: bot_id(bot),
            before: None,
        })
        .await;
    let mut all = Vec::new();
    loop {
        if let ToPhone::History {
            req: got,
            items,
            more,
            done,
            ..
        } = phone.next().await
        {
            assert_eq!(got, req);
            all.extend(items);
            if done {
                return (all, more);
            }
        }
    }
}

async fn chats(phone: &mut Phone) -> Vec<ChatLine> {
    phone.send(&FromPhone::Chats).await;
    let mut all = Vec::new();
    loop {
        if let ToPhone::Chats { bots, first } = phone.next().await {
            assert_eq!(first, all.is_empty());
            all.extend(bots);
            return all;
        }
    }
}

async fn send(phone: &mut Phone, bot: &Value, id: &str, text: &str) -> (bool, Option<String>) {
    phone
        .send(&FromPhone::Send {
            client_id: id.into(),
            bot_id: bot_id(bot),
            text: text.into(),
        })
        .await;
    loop {
        if let ToPhone::Sent {
            client_id,
            ok,
            reason,
        } = phone.next().await
        {
            assert_eq!(client_id, id);
            return (ok, reason);
        }
    }
}

#[tokio::test]
async fn the_list_has_the_active_bots_and_the_history_has_text_without_tool_output() {
    let (t, mut app, mut phone, bot) = start().await;
    let crew_id = app
        .call("crews.list", Value::Null)
        .await
        .expect("crews")
        .as_array()
        .and_then(|crews| crews.first().cloned())
        .expect("a crew")["id"]
        .clone();
    let gone = app
        .call(
            "bots.create",
            json!({ "crewId": crew_id, "name": "Old", "role": "x", "instructions": "" }),
        )
        .await
        .expect("second bot");
    app.call("bots.archive", json!({ "botId": gone["id"] }))
        .await
        .expect("archive");

    let list = chats(&mut phone).await;
    assert_eq!(list.len(), 1, "{list:?}");
    assert_eq!(
        (list[0].name.as_str(), list[0].crew.as_str()),
        ("Lead", "Ops")
    );
    assert_eq!(list[0].state, BotState::Idle);

    // The owner writes from the phone; the bot uses a tool and replies.
    assert_eq!(
        send(&mut phone, &bot, "c1", "Hello from here").await,
        (true, None)
    );
    let process = t.process_of(&bot).await;
    let lines = process.wait_lines(1).await;
    process.emit(stream::init("s1")).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    process
        .emit(stream::tool_use("t1", "Bash", json!({ "command": "ls" })))
        .await;
    process
        .emit(stream::tool_result("t1", "TOP-SECRET-OUTPUT", false))
        .await;
    process.emit(stream::text(&"é".repeat(ITEM_TEXT_MAX))).await;
    process.emit(stream::result(false)).await;

    let mut shown = Vec::new();
    for _ in 0..200 {
        shown = history(&mut phone, &bot, 1).await.0;
        if shown
            .iter()
            .any(|item| matches!(item, PhoneItem::Reply { .. }))
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let kinds: Vec<&str> = shown
        .iter()
        .map(|item| match item {
            PhoneItem::You { .. } => "you",
            PhoneItem::Tool { .. } => "tool",
            PhoneItem::Reply { .. } => "reply",
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        ["you", "tool", "reply"],
        "oldest first, no finished turn"
    );
    let wire = serde_json::to_string(&shown).expect("json");
    assert!(
        !wire.contains("TOP-SECRET-OUTPUT"),
        "a tool's output stays home"
    );
    assert!(
        matches!(&shown[0], PhoneItem::You { text, cut: false, .. } if text == "Hello from here")
    );
    match &shown[2] {
        PhoneItem::Reply { text, cut, .. } => {
            assert!(*cut && text.len() <= ITEM_TEXT_MAX, "{}", text.len());
        }
        other => panic!("{other:?}"),
    }
    // A page is one sealed message at most.
    assert!(wire.len() < SEALED_MAX * 2);
}

#[tokio::test]
async fn writing_goes_the_way_of_the_app_and_has_a_limit() {
    let (_t, mut app, mut phone, bot) = start().await;
    assert_eq!(
        send(&mut phone, &bot, "a", "  ").await.1.as_deref(),
        Some("invalid")
    );
    let nobody = json!({ "id": "bot_01ARZ3NDEKTSV4RRFFQ69G5FAV" });
    assert_eq!(
        send(&mut phone, &nobody, "b", "hi").await.1.as_deref(),
        Some("not_found")
    );

    // A bot that does everything without asking can be written to: the
    // phone has its own lock (spec 28.13).
    let crew_id = app.call("crews.list", Value::Null).await.expect("crews")[0]["id"].clone();
    let wild = app
        .call(
            "bots.create",
            json!({ "crewId": crew_id, "name": "Wild", "role": "x", "instructions": "",
                    "permissionMode": "bypass_permissions" }),
        )
        .await
        .expect("bot");
    assert_eq!(send(&mut phone, &wild, "w", "hi").await, (true, None));

    // 30 a minute, counting the three tries above.
    for n in 0..27 {
        assert!(
            send(&mut phone, &bot, &format!("m{n}"), "again").await.0,
            "{n}"
        );
    }
    let refused = send(&mut phone, &bot, "last", "too many").await;
    assert_eq!(refused, (false, Some("rate_limited".into())));
}

#[tokio::test]
async fn the_open_conversation_follows_the_reply_and_the_list_hears_of_it() {
    let (t, mut app, mut phone, bot) = start().await;
    let id = bot_id(&bot);
    phone
        .send(&FromPhone::Watch {
            bot_id: Some(id.clone()),
        })
        .await;
    assert!(matches!(
        phone.next().await,
        ToPhone::State {
            state: BotState::Idle,
            ..
        }
    ));

    app.call("messages.send", json!({ "botId": bot["id"], "body": "Go" }))
        .await
        .expect("send");
    let process = t.process_of(&bot).await;
    let lines = process.wait_lines(1).await;
    process.emit(stream::init("s1")).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    process.emit(stream::delta("Hel")).await;
    process.emit(stream::delta("lo")).await;
    process.emit(stream::text("Hello")).await;
    process.emit(stream::result(false)).await;

    let (mut lives, mut items, mut line) = (Vec::new(), Vec::new(), None);
    let replied = |items: &[PhoneItem]| {
        items
            .iter()
            .any(|item| matches!(item, PhoneItem::Reply { .. }))
    };
    while !(replied(&items) && line.is_some()) {
        match phone.next_any().await {
            ToPhone::Live { text, .. } => lives.push(text),
            ToPhone::Item { item, .. } => items.push(item),
            ToPhone::Line { bot } => line = Some(bot),
            _ => {}
        }
    }
    assert!(
        items
            .iter()
            .any(|item| matches!(item, PhoneItem::You { text, .. } if text == "Go"))
    );
    // The whole text so far, and no more than one a second.
    let written: Vec<&String> = lives.iter().filter(|text| !text.is_empty()).collect();
    assert!(
        written.len() == 1 && "Hello".starts_with(written[0].as_str()),
        "{lives:?}"
    );
    assert!(line.is_some_and(|line| line.last_reply_at.is_some() || line.last.is_some()));

    // Closed, the conversation stays quiet; the list still hears of replies.
    phone.send(&FromPhone::Watch { bot_id: None }).await;
    // What was still on its way from the first turn is read and dropped.
    t.until_state(&bot, BotState::Idle).await;
    chats(&mut phone).await;
    app.call(
        "messages.send",
        json!({ "botId": bot["id"], "body": "More" }),
    )
    .await
    .expect("send");
    let lines = process.wait_lines(2).await;
    t.clock.advance(Duration::from_secs(5));
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    process.emit(stream::delta("quiet")).await;
    process.emit(stream::text("Done")).await;
    process.emit(stream::result(false)).await;
    loop {
        match phone.next_any().await {
            ToPhone::Line { bot } => {
                assert_eq!(bot.name, "Lead");
                break;
            }
            ToPhone::Live { .. } | ToPhone::Item { .. } | ToPhone::State { .. } => {
                panic!("a closed conversation sent something")
            }
            _ => {}
        }
    }
}
