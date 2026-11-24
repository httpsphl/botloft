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

/// The first answer with one of `ids`, skipping notifications.
async fn first_answer(app: &mut common::Client, ids: &[i64]) -> i64 {
    loop {
        let frame = app.recv().await.expect("connection open");
        if let Some(id) = frame["id"].as_i64().filter(|id| ids.contains(id)) {
            return id;
        }
    }
}

#[tokio::test]
async fn sealing_or_opening_a_backup_does_not_hold_up_the_connection() {
    // A backup of big crew folders takes minutes; the app's ping must still
    // be answered meanwhile, or the app drops the connection (spec 14.2).
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    app.call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let export = json!({ "jsonrpc": "2.0", "id": 901, "method": "backup.export",
        "params": { "passphrase": "correct horse" } });
    app.send_raw(&export.to_string()).await;
    app.send_raw(r#"{"jsonrpc":"2.0","id":902,"method":"session.ping"}"#)
        .await;
    assert_eq!(first_answer(&mut app, &[901, 902]).await, 902);
    assert_eq!(first_answer(&mut app, &[901]).await, 901);

    let file = fs::read_dir(t.paths.home.join("exports"))
        .expect("exports")
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "botloft"))
        .expect("the sealed backup");
    let stage = json!({ "jsonrpc": "2.0", "id": 903, "method": "backup.stage",
        "params": { "path": file.display().to_string(), "passphrase": "correct horse" } });
    app.send_raw(&stage.to_string()).await;
    app.send_raw(r#"{"jsonrpc":"2.0","id":904,"method":"session.ping"}"#)
        .await;
    assert_eq!(first_answer(&mut app, &[903, 904]).await, 904);
    assert_eq!(first_answer(&mut app, &[903]).await, 903);
    app.call("backup.cancel", serde_json::Value::Null)
        .await
        .expect("cancel");
}
