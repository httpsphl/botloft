use botloft_core::protocol::Crew;

use super::*;
use crate::StoreError;

fn setup() -> (Store, CrewId) {
    let store = Store::open_in_memory().expect("store");
    let crew = Crew {
        id: CrewId::generate(),
        name: "Docs".to_owned(),
        slug: "docs".to_owned(),
        work_folder: String::new(),
        work_folder_chosen: false,
        lead_bot_id: None,
        paused: false,
        color: None,
        created_at: 1,
        archived_at: None,
    };
    store.insert_crew(&crew).expect("crew");
    (store, crew.id)
}

fn bot(crew: &CrewId, handle: &str, created_at: i64) -> BotRecord {
    BotRecord {
        id: BotId::generate(),
        crew_id: crew.clone(),
        name: handle.to_owned(),
        handle: handle.to_owned(),
        slug: handle.to_owned(),
        role: "writes docs".to_owned(),
        instructions: "Be brief.\nCite sources.".to_owned(),
        color: "#FF7A59".to_owned(),
        paused: false,
        permission_mode: PermissionMode::Default,
        model: BotModel::Default,
        model_in_use: None,
        effort: BotEffort::Default,
        effort_default: None,
        created_at,
        archived_at: None,
    }
}

#[test]
fn insert_get_and_list_by_crew() {
    let (store, crew) = setup();
    let writer = bot(&crew, "writer", 10);
    let editor = bot(&crew, "editor", 20);
    store.insert_bot(&writer).expect("insert");
    store.insert_bot(&editor).expect("insert");

    assert_eq!(store.bot(&writer.id).expect("get"), Some(writer.clone()));
    let expected = vec![writer, editor];
    assert_eq!(store.bots(Some(&crew), false).expect("list"), expected);
    assert_eq!(store.bots(None, false).expect("list"), expected);
    assert!(
        store
            .bots(Some(&CrewId::generate()), false)
            .expect("list")
            .is_empty()
    );
}

#[test]
fn bot_needs_an_existing_crew() {
    let (store, _) = setup();
    let orphan = store.insert_bot(&bot(&CrewId::generate(), "orphan", 1));
    assert!(matches!(orphan, Err(StoreError::Sqlite(_))), "{orphan:?}");
}

#[test]
fn handles_are_unique_among_active_bots_only() {
    let (store, crew) = setup();
    let mut first = bot(&crew, "writer", 1);
    store.insert_bot(&first).expect("insert");
    assert_eq!(
        store.active_bot_by_handle(&crew, "writer").expect("lookup"),
        Some(first.id.clone())
    );

    let mut clash = bot(&crew, "writer", 2);
    clash.slug = "writer-2".to_owned();
    assert!(matches!(
        store.insert_bot(&clash),
        Err(StoreError::Duplicate(_))
    ));

    first.archived_at = Some(3);
    store.update_bot(&first).expect("archive");
    assert_eq!(
        store.active_bot_by_handle(&crew, "writer").expect("lookup"),
        None
    );
    store.insert_bot(&clash).expect("handle is free again");
    assert!(store.bot_slug_exists(&crew, "writer").expect("slug"));
    assert_eq!(store.count_bots(&crew).expect("count"), 2);
}

#[test]
fn update_saves_mutable_fields() {
    let (store, crew) = setup();
    let mut writer = bot(&crew, "writer", 1);
    store.insert_bot(&writer).expect("insert");
    writer.name = "Lead Writer".to_owned();
    writer.handle = "lead-writer".to_owned();
    writer.color = "#5EC8FF".to_owned();
    writer.paused = true;
    writer.effort = BotEffort::Low;
    store.update_bot(&writer).expect("update");
    assert_eq!(store.bot(&writer.id).expect("get"), Some(writer));
}

#[test]
fn what_claude_code_reports_is_saved_apart_from_the_owners_choices() {
    let (store, crew) = setup();
    let mut writer = bot(&crew, "writer", 1);
    store.insert_bot(&writer).expect("insert");
    store
        .set_effort_default(&writer.id, Some(ModelEffort::Medium))
        .expect("report");
    // Saving the bot's own fields leaves the report alone.
    writer.role = "edits".to_owned();
    store.update_bot(&writer).expect("update");
    let saved = store.bot(&writer.id).expect("get").expect("bot");
    assert_eq!(saved.effort, BotEffort::Default);
    assert_eq!(saved.effort_default, Some(ModelEffort::Medium));

    store.set_effort_default(&writer.id, None).expect("forget");
    let saved = store.bot(&writer.id).expect("get").expect("bot");
    assert_eq!(saved.effort_default, None);
}

#[test]
fn archiving_the_crew_archives_its_bots() {
    let (store, crew) = setup();
    store.insert_bot(&bot(&crew, "writer", 1)).expect("insert");
    store.archive_crew(&crew, 9).expect("archive");
    assert!(store.bots(Some(&crew), false).expect("list").is_empty());
    let all = store.bots(Some(&crew), true).expect("list all");
    assert_eq!(all[0].archived_at, Some(9));
}
