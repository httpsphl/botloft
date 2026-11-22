//! Files in pieces for an app on another computer (spec 42.7): a bot's file
//! and a backup go out, a picked file comes in, and nothing else on the
//! daemon's computer can be read or written that way.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::CHUNK_MAX;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const CONFLICT: i64 = -32003;
const VALIDATION: i64 = -32004;

/// A crew with a bot; the bot and the crew's work folder.
async fn crew_with_bot(app: &mut Client) -> (Value, PathBuf) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let workspace = PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    let work = workspace.parent().expect("crew folder").join("shared");
    fs::create_dir_all(&work).expect("work folder");
    (bot, work)
}

/// Every piece of a file from `method`, as `params(offset)` asks for it.
async fn read_all(app: &mut Client, method: &str, params: impl Fn(u64) -> Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let piece = app
            .call(method, params(bytes.len() as u64))
            .await
            .expect("piece");
        let data = BASE64
            .decode(piece["data"].as_str().expect("data"))
            .expect("base64");
        assert!(data.len() as u64 <= CHUNK_MAX);
        bytes.extend(data);
        if bytes.len() as u64 >= piece["size"].as_u64().expect("size") {
            return bytes;
        }
    }
}

#[tokio::test]
async fn a_bots_file_goes_out_in_pieces() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let (bot, work) = crew_with_bot(&mut app).await;
    let report = work.join("report.bin");
    let bytes: Vec<u8> = (0..(CHUNK_MAX * 2 + 7))
        .map(|at| (at % 253) as u8)
        .collect();
    fs::write(&report, &bytes).expect("write");

    let read = read_all(
        &mut app,
        "files.chunk",
        |offset| json!({ "botId": bot["id"], "path": report, "offset": offset }),
    )
    .await;
    assert_eq!(read, bytes);

    let elsewhere = tempfile::tempdir().expect("dir");
    let private = elsewhere.path().join("private.txt");
    fs::write(&private, "private").expect("write");
    let refused = app
        .call(
            "files.chunk",
            json!({ "botId": bot["id"], "path": private, "offset": 0 }),
        )
        .await
        .expect_err("not one of the bot's files");
    assert_eq!(refused.code, NOT_FOUND);
}

#[tokio::test]
async fn only_a_backup_goes_out_from_the_exports() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let exported = app
        .call(
            "backup.export",
            json!({ "passphrase": "correct horse battery" }),
        )
        .await
        .expect("export");
    let path = exported["path"].as_str().expect("path").to_owned();
    let read = read_all(
        &mut app,
        "backup.chunk",
        |offset| json!({ "path": path, "offset": offset }),
    )
    .await;
    assert_eq!(read, fs::read(&path).expect("backup"));

    let db = t.paths.home.join("botloft.db");
    let climbing = Path::new(&path)
        .parent()
        .expect("exports")
        .join("..")
        .join("botloft.db");
    for refused in [db, climbing] {
        let err = app
            .call("backup.chunk", json!({ "path": refused, "offset": 0 }))
            .await
            .expect_err("not a backup");
        assert_eq!(err.code, NOT_FOUND, "{}", refused.display());
    }
}

#[tokio::test]
async fn a_picked_file_comes_in_piece_by_piece() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let started = app
        .call(
            "uploads.start",
            json!({ "name": "../../secrets/history.zip", "size": 5 }),
        )
        .await
        .expect("start");
    let id = started["uploadId"].clone();
    let piece = |offset: u64, text: &str| json!({ "uploadId": id, "offset": offset, "data": BASE64.encode(text) });

    app.call("uploads.chunk", piece(0, "abc"))
        .await
        .expect("first");
    let again = app
        .call("uploads.chunk", piece(0, "abc"))
        .await
        .expect_err("a piece sent twice");
    assert_eq!(again.code, CONFLICT);
    let early = app
        .call("uploads.finish", json!({ "uploadId": id }))
        .await
        .expect_err("not all of it yet");
    assert_eq!(early.code, CONFLICT);
    app.call("uploads.chunk", piece(3, "de"))
        .await
        .expect("second");
    let over = app
        .call("uploads.chunk", piece(5, "x"))
        .await
        .expect_err("past the size said");
    assert_eq!(over.code, VALIDATION);

    let done = app
        .call("uploads.finish", json!({ "uploadId": id }))
        .await
        .expect("finish");
    let path = PathBuf::from(done["path"].as_str().expect("path"));
    assert_eq!(fs::read(&path).expect("file"), b"abcde");
    assert_eq!(path.file_name().expect("name"), "history.zip");
    assert!(
        path.starts_with(t.paths.home.join("uploads")),
        "{}",
        path.display()
    );

    app.call("uploads.cancel", json!({ "uploadId": id }))
        .await
        .expect("cancel");
    assert!(!path.exists());
    let gone = app
        .call("uploads.finish", json!({ "uploadId": id }))
        .await
        .expect_err("cancelled");
    assert_eq!(gone.code, NOT_FOUND);
    let made_up = app
        .call(
            "uploads.chunk",
            json!({ "uploadId": "../secrets", "offset": 0, "data": "" }),
        )
        .await
        .expect_err("no such upload");
    assert_eq!(made_up.code, NOT_FOUND);
}

#[tokio::test]
async fn an_app_elsewhere_restarts_botloft_through_its_service() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    app.call("system.restart", Value::Null)
        .await
        .expect("the answer comes first");
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        t.daemon.restart_requested(),
    )
    .await
    .expect("a restart was asked for");
}
