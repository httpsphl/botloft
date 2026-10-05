//! Importing a backup (spec 14.2): staged with the passphrase while Botloft
//! runs, swapped in on the next start, with what was there moved aside.

mod common;

use std::fs;

use botloft_store::Store;
use botloftd::backup::restore;
use botloftd::paths::Paths;
use common::TestDaemon;
use serde_json::json;

#[tokio::test]
async fn a_backup_from_one_computer_replaces_another_after_a_restart() {
    // The computer the backup comes from.
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot");
    let memory = t.paths.bot_workspace("ops", "scout").join("CLAUDE.md");
    fs::write(&memory, "The client likes short reports.").expect("memory");
    let exported = app
        .call("backup.export", json!({ "passphrase": "correct horse" }))
        .await
        .expect("export");
    let file = std::path::PathBuf::from(exported["path"].as_str().expect("path"));

    // The other computer, with data of its own.
    let other = tempfile::tempdir().expect("tempdir");
    let paths = Paths::new(other.path().join("Data"), other.path().join("Bots"));
    fs::create_dir_all(&paths.home).expect("home");
    drop(Store::open(&paths.db()).expect("its own database"));
    fs::create_dir_all(paths.workspaces_root.join("old")).expect("old crew");
    fs::write(paths.workspaces_root.join("old").join("note.md"), "mine").expect("note");

    assert!(restore::stage(&paths, &file, "wrong horse").is_err());
    assert!(
        !paths.home.join("restore").exists(),
        "nothing kept from a failed try"
    );
    assert_eq!(
        restore::apply_pending(&paths, "t1").expect("nothing to do"),
        None,
        "nothing staged"
    );

    let manifest = restore::stage(&paths, &file, "correct horse").expect("stage");
    assert_eq!(manifest.crews[0].name, "Ops");
    assert_eq!(manifest.crews[0].bots, ["Scout"]);
    assert_eq!(
        restore::apply_pending(&paths, "t2").expect("not confirmed"),
        None,
        "staged but not confirmed"
    );
    restore::confirm(&paths).expect("confirm");

    let aside = restore::apply_pending(&paths, "t3")
        .expect("apply")
        .expect("applied");
    assert!(
        aside.join("botloft.db").is_file(),
        "the old database is kept"
    );
    let moved = other.path().join("Bots-before-restore-t3");
    assert_eq!(
        fs::read_to_string(moved.join("old").join("note.md")).expect("old note"),
        "mine",
        "the old folders are kept"
    );
    let store = Store::open(&paths.db()).expect("restored database");
    let crews = store.crews(true).expect("crews");
    assert_eq!(crews.len(), 1);
    assert_eq!(crews[0].name, "Ops");
    assert_eq!(
        fs::read_to_string(paths.bot_workspace("ops", "scout").join("CLAUDE.md")).expect("memory"),
        "The client likes short reports."
    );
    assert!(!paths.home.join("restore").exists());
    assert_eq!(
        restore::apply_pending(&paths, "t4").expect("once"),
        None,
        "applied once"
    );
}
