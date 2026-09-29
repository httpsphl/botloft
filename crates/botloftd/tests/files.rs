//! The files a bot made (spec 8.5): listed for the app's panel and read for
//! its preview, without opening the rest of the computer.

mod common;

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::BotState;
use common::{TestDaemon, stream};
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;

fn put(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    fs::write(path, text).expect("write");
}

fn names(files: &Value) -> Vec<String> {
    let mut names: Vec<String> = files
        .as_array()
        .expect("list")
        .iter()
        .map(|file| file["name"].as_str().expect("name").to_owned())
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn the_panel_lists_and_reads_what_the_bot_made() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    t.until_state(&bot, BotState::Idle).await;
    let process = t.process_of(&bot).await;

    let workspace = std::path::PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    let work = workspace.parent().expect("crew folder").join("shared");
    put(&work.join("report.pdf"), "pdf");
    put(&work.join("notes/a.md"), "# notes");
    put(&work.join(".hidden"), "x");
    put(&workspace.join("CLAUDE.md"), "memory");
    put(&workspace.join("attachments/2026-01-01/sent.png"), "png");
    // Older than the bot: the owner's own file, not something it made.
    put(&work.join("old.txt"), "old");
    let hour_ago = SystemTime::now() - Duration::from_secs(3600);
    fs::File::options()
        .write(true)
        .open(work.join("old.txt"))
        .expect("open")
        .set_modified(hour_ago)
        .expect("set mtime");

    // The bot writes a file far from its folders; another one it never touched.
    let elsewhere = tempfile::tempdir().expect("tempdir");
    put(&elsewhere.path().join("made.txt"), "made");
    put(&elsewhere.path().join("private.txt"), "private");
    let made = elsewhere.path().join("made.txt");
    let input = json!({ "file_path": made, "content": "made" });
    process
        .emit(stream::tool_use("toolu_1", "Write", input))
        .await;

    let list = json!({ "botId": bot["id"] });
    let mut files = Value::Null;
    for _ in 0..100 {
        files = app.call("files.list", list.clone()).await.expect("list");
        if names(&files).contains(&"made.txt".to_owned()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(names(&files), ["a.md", "made.txt", "report.pdf"]);
    let made_entry = files
        .as_array()
        .expect("list")
        .iter()
        .find(|file| file["name"] == "made.txt")
        .expect("made");
    assert_eq!(made_entry["writtenByBot"], true);
    let notes = files
        .as_array()
        .expect("list")
        .iter()
        .find(|file| file["name"] == "a.md")
        .expect("notes");
    assert_eq!(notes["folder"], "notes");
    assert_eq!(notes["mediaType"], "text/markdown");
    assert_eq!(notes["writtenByBot"], false);

    let read = |path: &Path| json!({ "botId": bot["id"], "path": path });
    let pdf = app
        .call("files.read", read(&work.join("report.pdf")))
        .await
        .expect("read");
    assert_eq!(
        pdf,
        json!({ "mediaType": "application/pdf", "data": BASE64.encode(b"pdf") })
    );
    app.call("files.read", read(&made)).await.expect("written");

    // Nothing else on the computer opens through the panel.
    let private = elsewhere.path().join("private.txt");
    // Two levels up from the work folder is the folder of every crew.
    put(&work.join("..").join("..").join("secret.txt"), "secret");
    let climbing = work.join("..").join("..").join("secret.txt");
    for refused in [
        private,
        climbing,
        Path::new("report.pdf").to_path_buf(),
        work.join("missing.txt"),
        work.join("notes"),
    ] {
        let err = app
            .call("files.read", read(&refused))
            .await
            .expect_err(&format!("{} is refused", refused.display()));
        assert_eq!(err.code, NOT_FOUND, "{}", refused.display());
    }
}
