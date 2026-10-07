//! The light copy that goes to the cloud (spec 14.2, 27): the structure, the
//! bots' memory and the routines, and not the conversations or the files.

mod common;

use std::fs::{self, File};
use std::io::Read;

use botloft_store::Store;
use botloftd::backup::{restore, seal};
use botloftd::paths::Paths;
use common::TestDaemon;
use serde_json::json;

#[tokio::test]
async fn the_light_copy_carries_memory_and_routines_but_not_chats_or_files() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let folder = t.paths.bot_workspace("ops", "scout");
    fs::write(folder.join("CLAUDE.md"), "The client likes short reports.").expect("memory");
    fs::create_dir_all(folder.join("attachments").join("2026-10-06")).expect("attachments");
    fs::write(
        folder.join("attachments").join("2026-10-06").join("a.png"),
        "png",
    )
    .expect("a");
    fs::create_dir_all(folder.join("node_modules")).expect("modules");
    fs::write(folder.join("node_modules").join("x.js"), "x").expect("x");
    fs::write(t.paths.shared_dir("ops").join("report.md"), "# Report").expect("report");
    app.call(
        "routines.create",
        json!({
            "botId": bot["id"], "name": "Morning", "prompt": "Summarize.",
            "schedule": { "kind": "interval", "minutes": 60 }, "timezone": "UTC"
        }),
    )
    .await
    .expect("routine");
    app.call(
        "messages.send",
        json!({ "botId": bot["id"], "body": "A private sentence from the chat." }),
    )
    .await
    .expect("message");

    // A full export first: the light one must not take its place.
    let full = app
        .call("backup.export", json!({ "passphrase": "correct horse" }))
        .await
        .expect("full");
    assert_eq!(full["manifest"]["scope"], "full");
    let light = app
        .call(
            "backup.export",
            json!({ "passphrase": "correct horse", "scope": "light" }),
        )
        .await
        .expect("light");
    assert_eq!(light["manifest"]["scope"], "light");
    assert_eq!(light["manifest"]["crews"][0]["bots"], json!(["Scout"]));
    assert!(fs::metadata(full["path"].as_str().expect("path")).is_ok());
    assert!(
        light["path"]
            .as_str()
            .expect("path")
            .contains("cloud-upload")
    );
    assert!(light["size"].as_u64().expect("size") < full["size"].as_u64().expect("size"));

    let path = light["path"].as_str().expect("path");
    let mut zip_bytes = Vec::new();
    seal::open(
        &mut File::open(path).expect("open"),
        &mut zip_bytes,
        "correct horse",
    )
    .expect("unseal");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).expect("zip");
    let names: Vec<String> = zip.file_names().map(str::to_owned).collect();
    for wanted in [
        "manifest.json",
        "botloft.db",
        "workspaces/ops/scout/CLAUDE.md",
        "workspaces/ops/scout/.claude/rules/botloft.md",
    ] {
        assert!(
            names.iter().any(|name| name == wanted),
            "{wanted} in {names:?}"
        );
    }
    for unwanted in [
        "attachments",
        "node_modules",
        "shared",
        "settings.json",
        ".botloft",
    ] {
        assert!(
            !names.iter().any(|name| name.contains(unwanted)),
            "{unwanted} in {names:?}"
        );
    }
    let mut database = Vec::new();
    zip.by_name("botloft.db")
        .expect("database")
        .read_to_end(&mut database)
        .expect("read");
    assert!(
        !String::from_utf8_lossy(&database).contains("A private sentence"),
        "no conversation in the light copy"
    );

    // Another computer restores it: the crew, the bot, its memory, the routine.
    let other = tempfile::tempdir().expect("tempdir");
    let paths = Paths::new(other.path().join("Data"), other.path().join("Bots"));
    fs::create_dir_all(&paths.home).expect("home");
    drop(Store::open(&paths.db()).expect("its own database"));
    let manifest =
        restore::stage(&paths, std::path::Path::new(path), "correct horse").expect("stage");
    assert_eq!(manifest.scope, botloft_core::protocol::BackupScope::Light);
    restore::confirm(&paths).expect("confirm");
    restore::apply_pending(&paths, "t1")
        .expect("apply")
        .expect("applied");
    let store = Store::open(&paths.db()).expect("restored database");
    assert_eq!(store.crews(true).expect("crews")[0].name, "Ops");
    let bots = store.bots(None, true).expect("bots");
    assert_eq!(bots.len(), 1);
    assert_eq!(store.routines(None, true).expect("routines").len(), 1);
    assert_eq!(
        fs::read_to_string(paths.bot_workspace("ops", "scout").join("CLAUDE.md")).expect("memory"),
        "The client likes short reports."
    );
}

#[tokio::test]
async fn a_backup_from_before_the_scope_existed_reads_as_full() {
    let manifest = r#"{"format":1,"createdAt":1,"version":"0.10.0","crews":[]}"#;
    let parsed: botloft_core::protocol::BackupManifest =
        serde_json::from_str(manifest).expect("manifest");
    assert_eq!(parsed.scope, botloft_core::protocol::BackupScope::Full);
}
