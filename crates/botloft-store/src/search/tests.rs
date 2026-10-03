//! Tests for searching the chats.

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{ChatBody, ChatItem, ReplyItem, ToolItem, ToolStatus};

use super::*;
use crate::tests::{Fixture, message_to};

fn reply(store: &Store, bot: &BotId, text: &str) -> ChatItem {
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body: ChatBody::Reply(ReplyItem { text: text.into() }),
        created_at: 0,
        updated_at: 0,
    };
    store.insert_chat_item(&item).expect("reply");
    item
}

fn find(store: &Store, words: &str, filter: SearchFilter<'_>) -> Vec<Found> {
    let query = fts_query(words).expect("words");
    store
        .search_chat(
            &query,
            SearchFilter {
                limit: 50,
                ..filter
            },
        )
        .expect("search")
}

fn texts(found: &[Found]) -> Vec<&str> {
    found
        .iter()
        .map(|hit| match &hit.item.body {
            ChatBody::Reply(reply) => reply.text.as_str(),
            ChatBody::Inbound(inbound) => inbound.message.body.as_str(),
            _ => "",
        })
        .collect()
}

#[test]
fn words_are_found_in_any_order_without_accents_and_by_their_start() {
    let fx = Fixture::new();
    let scout = &fx.bots[0].id;
    reply(&fx.store, scout, "A previsão do tempo para sábado");
    reply(&fx.store, scout, "Nothing about the weather here");
    let (note, delivery) = message_to(&fx.crew.id, scout, "Check the weather for Saturday");
    fx.store
        .insert_message(&note, &delivery, None)
        .expect("note");

    assert_eq!(
        texts(&find(&fx.store, "sabado previsao", SearchFilter::default())),
        ["A previsão do tempo para sábado"]
    );
    // Newest first, and the last word may be cut short.
    assert_eq!(
        texts(&find(&fx.store, "weath", SearchFilter::default())),
        [
            "Check the weather for Saturday",
            "Nothing about the weather here"
        ]
    );
    let hit = &find(&fx.store, "saturday", SearchFilter::default())[0];
    assert_eq!(hit.crew, fx.crew.id);
    assert!(
        hit.snippet.contains("\u{2}Saturday\u{3}"),
        "{:?}",
        hit.snippet
    );
}

#[test]
fn filters_paging_and_what_is_left_out() {
    let fx = Fixture::new();
    let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
    let first = reply(&fx.store, scout, "report one");
    let second = reply(&fx.store, scout, "report two");
    reply(&fx.store, writer, "report three");
    // Tool calls are not searched.
    let tool = ChatItem {
        id: ChatItemId::generate(),
        bot_id: scout.clone(),
        body: ChatBody::Tool(ToolItem {
            tool_use_id: "toolu_1".into(),
            name: "Bash".into(),
            summary: "report".into(),
            explanation: Some("report".into()),
            input: "{\"command\":\"report\"}".into(),
            status: ToolStatus::Done,
            output: Some("report".into()),
            file: None,
        }),
        created_at: 0,
        updated_at: 0,
    };
    fx.store.insert_chat_item(&tool).expect("tool");

    let only_scout = SearchFilter {
        bot: Some(scout),
        ..SearchFilter::default()
    };
    assert_eq!(
        texts(&find(&fx.store, "report", only_scout)),
        ["report two", "report one"]
    );
    let crew = SearchFilter {
        crew: Some(&fx.crew.id),
        ..SearchFilter::default()
    };
    assert_eq!(find(&fx.store, "report", crew).len(), 3);
    let older = SearchFilter {
        before: Some(&second.id),
        ..SearchFilter::default()
    };
    let page = find(&fx.store, "report", older);
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].item.id, first.id);

    let mut archived = fx.bots[1].clone();
    archived.archived_at = Some(5);
    fx.store.update_bot(&archived).expect("archive");
    assert_eq!(find(&fx.store, "report", SearchFilter::default()).len(), 2);
}

#[test]
fn the_index_follows_changes_and_deletes() {
    let fx = Fixture::new();
    let scout = &fx.bots[0].id;
    let item = reply(&fx.store, scout, "draft about apples");
    fx.store
        .update_chat_item(
            &item.id,
            &ChatBody::Reply(ReplyItem {
                text: "final about pears".into(),
            }),
            1,
        )
        .expect("update");
    assert!(find(&fx.store, "apples", SearchFilter::default()).is_empty());
    assert_eq!(find(&fx.store, "pears", SearchFilter::default()).len(), 1);

    assert!(fx.store.delete_bot(scout).expect("delete"));
    assert!(find(&fx.store, "pears", SearchFilter::default()).is_empty());
    let integrity: rusqlite::Result<()> = fx
        .store
        .conn
        .execute_batch("INSERT INTO chat_search (chat_search, rank) VALUES ('integrity-check', 1)");
    assert!(integrity.is_ok(), "{integrity:?}");
}

#[test]
fn chat_since_opens_the_chat_at_an_item() {
    let fx = Fixture::new();
    let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
    reply(&fx.store, scout, "zero");
    let one = reply(&fx.store, scout, "one");
    reply(&fx.store, scout, "two");
    let since = fx.store.chat_since(scout, &one.id, 100).expect("since");
    assert_eq!(since.len(), 2);
    assert_eq!(since[1].id, one.id);
    assert!(
        fx.store
            .chat_since(writer, &one.id, 100)
            .expect("other")
            .is_empty()
    );
}

#[test]
fn queries_take_words_literally() {
    assert_eq!(fts_query("  "), None);
    assert_eq!(fts_query("- ?"), None);
    assert_eq!(fts_query("weather"), Some("\"weather\"*".into()));
    assert_eq!(
        fts_query("say \"NEAR\" OR x*"),
        Some("\"say\" \"NEAR\" \"OR\" \"x*\"*".into())
    );
}
