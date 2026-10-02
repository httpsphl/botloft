use botloft_core::protocol::{ActivityKind, ReplyItem, ToolItem, ToolStatus, TurnItem};

use super::*;
use crate::tests::Fixture;

fn item(bot: &BotId, body: ChatBody, at: i64) -> ChatItem {
    ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body,
        created_at: at,
        updated_at: at,
    }
}

fn reply(text: &str) -> ChatBody {
    ChatBody::Reply(ReplyItem { text: text.into() })
}

#[test]
fn items_page_back_newest_first() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    let other = &fx.bots[1].id;
    let ids: Vec<_> = (0..3)
        .map(|n| {
            let entry = item(bot, reply(&format!("r{n}")), 10 + n);
            fx.store.insert_chat_item(&entry).expect("insert");
            entry.id
        })
        .collect();
    fx.store
        .insert_chat_item(&item(other, reply("elsewhere"), 20))
        .expect("insert");
    let newest = fx.store.chat_history(bot, None, 2).expect("history");
    assert_eq!(
        newest.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
        [ids[2].clone(), ids[1].clone()]
    );
    let older = fx
        .store
        .chat_history(bot, Some(&ids[1]), 10)
        .expect("older");
    assert_eq!(older.len(), 1);
    assert_eq!(older[0].body, reply("r0"));
}

#[test]
fn a_tool_item_is_found_and_updated() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    let running = ToolItem {
        tool_use_id: "toolu_7".into(),
        name: "Bash".into(),
        summary: "npm test".into(),
        explanation: None,
        input: "{}".into(),
        status: ToolStatus::Running,
        output: None,
        file: None,
    };
    let entry = item(bot, ChatBody::Tool(running.clone()), 5);
    fx.store.insert_chat_item(&entry).expect("insert");
    let found = fx
        .store
        .tool_item(bot, "toolu_7")
        .expect("find")
        .expect("some");
    assert_eq!(found.id, entry.id);
    assert!(fx.store.tool_item(bot, "toolu_8").expect("find").is_none());

    let done = ChatBody::Tool(ToolItem {
        status: ToolStatus::Done,
        output: Some("ok".into()),
        ..running
    });
    let updated = fx
        .store
        .update_chat_item(&entry.id, &done, 9)
        .expect("update")
        .expect("some");
    assert_eq!(
        (updated.body, updated.created_at, updated.updated_at),
        (done, 5, 9)
    );
}

#[test]
fn written_files_come_newest_first_without_repeats_or_failures() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    let write = |file: Option<&str>, status: ToolStatus| {
        ChatBody::Tool(ToolItem {
            tool_use_id: "t".into(),
            name: "Write".into(),
            summary: String::new(),
            explanation: None,
            input: "{}".into(),
            status,
            output: None,
            file: file.map(str::to_owned),
        })
    };
    for (n, (file, status)) in [
        (Some(r"C:\work\a.md"), ToolStatus::Done),
        (Some(r"C:\work\b.pdf"), ToolStatus::Done),
        (Some(r"C:\work\bad.md"), ToolStatus::Failed),
        (None, ToolStatus::Done),
        (Some(r"C:\work\a.md"), ToolStatus::Done),
    ]
    .into_iter()
    .enumerate()
    {
        fx.store
            .insert_chat_item(&item(bot, write(file, status), n as i64))
            .expect("insert");
    }
    let files = fx.store.written_files(bot, 10).expect("files");
    assert_eq!(files, [r"C:\work\a.md", r"C:\work\b.pdf"]);
    assert_eq!(
        fx.store.written_files(bot, 1).expect("one"),
        [r"C:\work\a.md"]
    );
}

#[test]
fn the_last_activity_skips_turn_ends() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    assert_eq!(fx.store.last_activity(bot).expect("none"), None);
    fx.store
        .insert_chat_item(&item(bot, reply("All done.\nDetails below"), 7))
        .expect("insert");
    let turn = ChatBody::Turn(TurnItem {
        duration_ms: 1000,
        tokens: None,
        error: None,
    });
    fx.store
        .insert_chat_item(&item(bot, turn, 8))
        .expect("insert");
    assert_eq!(
        fx.store.last_activity(bot).expect("activity"),
        Some(Activity {
            kind: ActivityKind::Reply,
            text: "All done. Details below".into(),
            tool: None,
            at: 7
        })
    );
}

#[test]
fn the_last_reply_is_found_behind_newer_items() {
    let fx = Fixture::new();
    let bot = &fx.bots[0].id;
    let other = &fx.bots[1].id;
    assert_eq!(fx.store.last_reply_at(bot).expect("none"), None);
    for (body, at) in [(reply("First."), 5), (reply("Second."), 9)] {
        fx.store
            .insert_chat_item(&item(bot, body, at))
            .expect("insert");
    }
    let turn = ChatBody::Turn(TurnItem {
        duration_ms: 1000,
        tokens: None,
        error: None,
    });
    fx.store
        .insert_chat_item(&item(bot, turn, 12))
        .expect("insert");
    fx.store
        .insert_chat_item(&item(other, reply("Elsewhere."), 20))
        .expect("insert");
    assert_eq!(fx.store.last_reply_at(bot).expect("reply"), Some(9));
}
