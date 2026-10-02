//! `share_file` (spec 10): the bot shows the owner its files as cards in
//! the chat, and only files the app may read.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::TestDaemon;
use common::mcp::Mcp;
use serde_json::{Value, json};

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    fs::write(path, text).expect("write");
}

/// A running bot, its MCP client and its two folders: its own and the crew's.
async fn one_bot() -> (TestDaemon, Mcp, PathBuf, PathBuf) {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let process = t.process_of(&bot).await;
    let client = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    let workspace = PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    let work = workspace.parent().expect("crew folder").join("shared");
    (t, client, workspace, work)
}

#[tokio::test]
async fn a_bot_shares_files_from_its_folders() {
    let (_t, mut client, workspace, work) = one_bot().await;
    put(&work.join("report.pdf"), "pdf!");
    put(&workspace.join("out/chart.png"), "png");

    let shared = client
        .tool(
            "share_file",
            json!({ "files": [work.join("report.pdf"), "out/chart.png"] }),
        )
        .await
        .expect("shared");
    let shown = shared["shown"].as_array().expect("shown");
    assert_eq!(shown.len(), 2);
    assert_eq!(shown[0]["name"], "report.pdf");
    assert_eq!(shown[0]["mediaType"], "application/pdf");
    assert_eq!(shown[0]["size"], 4);
    assert_eq!(shown[0]["folder"], "");
    // A relative path is in the bot's own folder, where it runs.
    assert_eq!(shown[1]["name"], "chart.png");
    assert_eq!(shown[1]["folder"], "out");
    let chart = PathBuf::from(shown[1]["path"].as_str().expect("path"));
    assert!(chart.is_absolute(), "{}", chart.display());
    assert!(chart.ends_with(Path::new("out").join("chart.png")));
}

#[tokio::test]
async fn nothing_outside_its_folders_is_shared() {
    let (_t, mut client, _workspace, work) = one_bot().await;
    put(&work.join("ok.txt"), "ok");
    let elsewhere = tempfile::tempdir().expect("tempdir");
    let private = elsewhere.path().join("private.txt");
    put(&private, "private");

    let outside = client
        .tool(
            "share_file",
            json!({ "files": [work.join("ok.txt"), private] }),
        )
        .await
        .expect_err("outside");
    assert!(
        outside.contains("copy it into the work folder"),
        "{outside}"
    );

    for (files, says) in [
        (json!([work.join("missing.txt")]), "There is no file"),
        (json!([work]), "There is no file"),
        (json!([]), "1 to 10"),
    ] {
        let err = client
            .tool("share_file", json!({ "files": files }))
            .await
            .expect_err("refused");
        assert!(err.contains(says), "{err}");
    }
    let unknown: Value = json!({ "files": ["ok.txt"], "caption": "x" });
    let err = client.tool("share_file", unknown).await.expect_err("args");
    assert!(err.contains("invalid arguments"), "{err}");
}
