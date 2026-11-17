//! The history of a bot's memory and instructions (spec 8.10): each version
//! with who made it, and going back to one, which is itself a new version.

mod common;

use std::fs;
use std::time::Duration;

use botloft_core::protocol::{
    BotsUpdateParams, Missed, Overlap, VersionAuthor, VersionKind, VersionsListParams,
    VersionsRestoreParams,
};
use botloftd::service::{bots, versions as service};
use botloftd::versions::{self, Origin};
use common::routines::*;
use common::stream;

fn history(s: &common::supervised::Setup, kind: VersionKind) -> Vec<(VersionAuthor, String)> {
    service::list(
        &s.daemon,
        VersionsListParams {
            bot_id: s.bot.clone(),
            kind,
        },
    )
    .expect("list")
    .into_iter()
    .map(|version| (version.author, version.text))
    .collect()
}

#[tokio::test(start_paused = true)]
async fn memory_changes_keep_who_made_them_and_can_be_undone() {
    let s = routines_setup().await;
    let memory = versions::memory_path(&s.daemon, &s.bot).expect("memory file");
    // The owner wrote it before Botloft ever looked: the start.
    fs::write(&memory, "Client: Acme.\n").expect("write");

    // A turn: what changes in it is the bot's.
    create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    fs::write(&memory, "Client: Acme.\nPrefers short reports.\n").expect("bot writes");
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Between turns, someone else changes it.
    fs::write(&memory, "Client: Acme Ltd.\nPrefers short reports.\n").expect("outside");
    assert_eq!(
        history(&s, VersionKind::Memory),
        [
            (
                VersionAuthor::Outside,
                "Client: Acme Ltd.\nPrefers short reports.\n".to_owned()
            ),
            (
                VersionAuthor::Agent,
                "Client: Acme.\nPrefers short reports.\n".to_owned()
            ),
            (VersionAuthor::Start, "Client: Acme.\n".to_owned()),
        ]
    );

    // The owner goes back to the start: the file and a new version say so.
    let start = service::list(
        &s.daemon,
        VersionsListParams {
            bot_id: s.bot.clone(),
            kind: VersionKind::Memory,
        },
    )
    .expect("list")
    .pop()
    .expect("start");
    let back = service::restore(
        &s.daemon,
        VersionsRestoreParams {
            version_id: start.id.clone(),
        },
    )
    .expect("restore");
    assert_eq!(back.author, VersionAuthor::Owner);
    assert_eq!(back.restored_from, Some(start.id));
    assert_eq!(
        fs::read_to_string(&memory).expect("read"),
        "Client: Acme.\n"
    );
    assert_eq!(
        history(&s, VersionKind::Memory).len(),
        4,
        "the undone one stays"
    );
}

#[tokio::test(start_paused = true)]
async fn instructions_keep_who_changed_them_and_can_be_undone() {
    let s = routines_setup().await;
    let change = |text: &str| BotsUpdateParams {
        bot_id: s.bot.clone(),
        name: None,
        role: None,
        instructions: Some(text.into()),
        color: None,
        allowed_commands: None,
        plan_first: None,
    };
    bots::update(&s.daemon, change("Write in English.")).expect("owner");
    bots::update_by(
        &s.daemon,
        change("Write in English. Keep it short."),
        Origin::by(VersionAuthor::Agent),
    )
    .expect("agent");
    // A name alone leaves the history as it was.
    bots::update(
        &s.daemon,
        BotsUpdateParams {
            name: Some("Scout".into()),
            instructions: None,
            ..change("")
        },
    )
    .expect("rename");
    assert_eq!(
        history(&s, VersionKind::Instructions),
        [
            (
                VersionAuthor::Agent,
                "Write in English. Keep it short.".to_owned()
            ),
            (VersionAuthor::Owner, "Write in English.".to_owned()),
        ]
    );

    let owners = service::list(
        &s.daemon,
        VersionsListParams {
            bot_id: s.bot.clone(),
            kind: VersionKind::Instructions,
        },
    )
    .expect("list")
    .pop()
    .expect("owner's");
    let back = service::restore(
        &s.daemon,
        VersionsRestoreParams {
            version_id: owners.id.clone(),
        },
    )
    .expect("restore");
    assert_eq!(back.restored_from, Some(owners.id));
    let record = s.daemon.store().bot(&s.bot).expect("read").expect("bot");
    assert_eq!(record.instructions, "Write in English.");
}
