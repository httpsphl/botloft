//! Exporting a backup (spec 14.2): one file, sealed with the owner's
//! passphrase, with the database and every crew's folder, and without what
//! Botloft writes again on every start.

mod common;

use std::fs::{self, File};
use std::io::Read;

use botloftd::backup::seal;
use common::TestDaemon;
use serde_json::{Value, json};

#[tokio::test]
async fn a_backup_holds_the_database_and_the_crews_folders_sealed() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot");
    let memory = t.paths.bot_workspace("ops", "scout").join("CLAUDE.md");
    fs::write(&memory, "Remember the client likes short reports.").expect("memory");
    fs::write(t.paths.shared_dir("ops").join("report.md"), "# Report").expect("report");

    let short = app
        .call("backup.export", json!({ "passphrase": "short" }))
        .await
        .expect_err("short");
    assert!(short.message.contains("at least 8"), "{}", short.message);

    let exported = app
        .call("backup.export", json!({ "passphrase": "correct horse" }))
        .await
        .expect("export");
    assert_eq!(exported["manifest"]["crews"][0]["name"], "Ops");
    assert_eq!(exported["manifest"]["crews"][0]["bots"], json!(["Scout"]));
    let path = exported["path"].as_str().expect("path");
    assert!(path.ends_with(".botloft"), "{path}");
    let sealed = fs::read(path).expect("file");
    assert_eq!(exported["size"], sealed.len());
    assert!(
        !String::from_utf8_lossy(&sealed).contains("short reports"),
        "nothing readable without the passphrase"
    );

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
        "workspaces/ops/shared/report.md",
        "workspaces/ops/scout/.claude/rules/botloft.md",
    ] {
        assert!(
            names.iter().any(|name| name == wanted),
            "{wanted} in {names:?}"
        );
    }
    assert!(
        !names
            .iter()
            .any(|name| name.contains(".botloft/") || name.ends_with(".claude/settings.json")),
        "{names:?}"
    );
    let mut text = String::new();
    zip.by_name("workspaces/ops/scout/CLAUDE.md")
        .expect("memory")
        .read_to_string(&mut text)
        .expect("read");
    assert_eq!(text, "Remember the client likes short reports.");
    let mut manifest = String::new();
    zip.by_name("manifest.json")
        .expect("manifest")
        .read_to_string(&mut manifest)
        .expect("read");
    let manifest: Value = serde_json::from_str(&manifest).expect("json");
    assert_eq!(manifest["format"], 1);

    let wrong = seal::open(
        &mut File::open(path).expect("open"),
        &mut Vec::new(),
        "wrong horse",
    );
    assert!(wrong.is_err());
}
