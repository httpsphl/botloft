//! Tests of the `conversations` table.

use super::*;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("dir");
    let store = Store::open(&dir.path().join("a.db")).expect("store");
    store
        .conn
        .execute_batch(
            "INSERT INTO crews (id, name, slug, created_at) VALUES ('crw_1', 'Ops', 'ops', 1);
             INSERT INTO bots (id, crew_id, name, handle, slug, role, instructions, color, \
              created_at)
              VALUES ('bot_01J9Z3K8000000000000000000', 'crw_1', 'Scout', 'scout', 'scout', \
              'r', 'i', 'c', 1);",
        )
        .expect("rows");
    (dir, store)
}

#[test]
fn the_main_conversation_holds_the_session_of_the_bot() {
    let (_dir, store) = store();
    let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
    assert_eq!(store.session_id(&bot).unwrap(), None);
    store.set_session_id(&bot, Some("session-1")).unwrap();
    assert_eq!(
        store.session_id(&bot).unwrap().as_deref(),
        Some("session-1")
    );
    let main = store
        .conversation(&ConversationId::main_of(&bot))
        .unwrap()
        .expect("main");
    assert!(main.main);
    assert_eq!(main.bot_id, bot);
}

#[test]
fn other_conversations_keep_their_own_session() {
    let (_dir, store) = store();
    let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
    store.set_session_id(&bot, Some("main-session")).unwrap();
    let other = ConversationRecord::new(&bot, 5);
    store.insert_conversation(&other).unwrap();
    store
        .set_conversation_session(&bot, &other.id, Some("other-session"))
        .unwrap();
    assert_eq!(
        store.session_id(&bot).unwrap().as_deref(),
        Some("main-session")
    );
    assert_eq!(
        store.conversation_session(&other.id).unwrap().as_deref(),
        Some("other-session")
    );
    let listed: Vec<bool> = store
        .conversations(Some(&bot))
        .unwrap()
        .iter()
        .map(|each| each.main)
        .collect();
    assert_eq!(listed, [true, false]);
}

#[test]
fn a_conversation_is_named_archived_and_brought_back() {
    let (_dir, store) = store();
    let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
    store.set_session_id(&bot, None).unwrap();
    let other = ConversationRecord::new(&bot, 5);
    store.insert_conversation(&other).unwrap();
    store
        .set_conversation_title(&other.id, Some("Trip plans"), Some("owner"))
        .unwrap();
    let named = store.conversation(&other.id).unwrap().expect("named");
    assert_eq!(named.title.as_deref(), Some("Trip plans"));
    assert_eq!(named.title_by.as_deref(), Some("owner"));

    store.set_conversation_archived(&other.id, Some(9)).unwrap();
    assert_eq!(store.conversations(Some(&bot)).unwrap().len(), 1);
    let archived = store.archived_conversations(&bot).unwrap();
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].archived_at, Some(9));

    store.set_conversation_archived(&other.id, None).unwrap();
    assert!(store.archived_conversations(&bot).unwrap().is_empty());
    assert_eq!(store.conversations(Some(&bot)).unwrap().len(), 2);
    assert_eq!(store.conversation_activity(&other.id).unwrap(), None);
}
