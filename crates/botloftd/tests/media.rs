//! Sounds and videos the app plays (spec 8.4): `files.media` links one of
//! the bot's files, and `/media/<token>` serves it whole or in the pieces a
//! player asks for, and nothing else.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::TestDaemon;
use common::bots::ready_bot;
use reqwest::StatusCode;
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const VALIDATION: i64 = -32004;

fn put(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
    fs::write(path, bytes).expect("write");
}

fn header(response: &reqwest::Response, name: reqwest::header::HeaderName) -> String {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

async fn get(url: &str, range: Option<&str>) -> reqwest::Response {
    let mut request = reqwest::Client::new().get(url);
    if let Some(range) = range {
        request = request.header(RANGE, range);
    }
    request.send().await.expect("request")
}

#[tokio::test]
async fn a_bots_video_plays_whole_or_in_pieces_and_nothing_else_does() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Studio" }))
        .await
        .expect("crew");
    let (bot, _, _) = ready_bot(&t, &mut app, &crew, "Editor").await;
    let workspace = PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    let work = workspace.parent().expect("crew folder").join("shared");

    // Bigger than one piece read from disk, so it streams in several.
    let video: Vec<u8> = (0..700_000u32).map(|n| (n % 251) as u8).collect();
    let clip = work.join("cuts").join("clip.mp4");
    put(&clip, &video);
    put(&work.join("notes.txt"), b"notes");
    let elsewhere = tempfile::tempdir().expect("tempdir");
    put(&elsewhere.path().join("private.mp3"), b"private");

    let link = |path: &Path| json!({ "botId": bot["id"], "path": path });
    let answer = app.call("files.media", link(&clip)).await.expect("media");
    let given = answer["url"].as_str().expect("url").to_owned();
    let again = app.call("files.media", link(&clip)).await.expect("media");
    assert_eq!(
        again["url"],
        Value::String(given.clone()),
        "one link per file"
    );
    // The address carries the daemon's configured port; the test one
    // listens on another.
    let path = given
        .strip_prefix("http://127.0.0.1:")
        .and_then(|rest| rest.find('/').map(|slash| &rest[slash..]))
        .expect("an address on the daemon");
    assert!(path.starts_with("/media/"), "{given}");
    let url = format!("http://{}{path}", t.addr);

    let whole = get(&url, None).await;
    assert_eq!(whole.status(), StatusCode::OK);
    assert_eq!(header(&whole, CONTENT_TYPE), "video/mp4");
    assert_eq!(header(&whole, ACCEPT_RANGES), "bytes");
    assert_eq!(header(&whole, CONTENT_LENGTH), "700000");
    assert_eq!(
        whole.bytes().await.expect("body").as_ref(),
        video.as_slice()
    );

    let piece = get(&url, Some("bytes=300000-300009")).await;
    assert_eq!(piece.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(header(&piece, CONTENT_RANGE), "bytes 300000-300009/700000");
    assert_eq!(
        piece.bytes().await.expect("body").as_ref(),
        &video[300_000..300_010]
    );

    let rest = get(&url, Some("bytes=699990-")).await;
    assert_eq!(rest.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        rest.bytes().await.expect("body").as_ref(),
        &video[699_990..]
    );

    let past = get(&url, Some("bytes=700000-")).await;
    assert_eq!(past.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(header(&past, CONTENT_RANGE), "bytes */700000");

    // Only sounds and videos get a link, and only the bot's own.
    let err = app
        .call("files.media", link(&work.join("notes.txt")))
        .await
        .expect_err("a text file does not play");
    assert_eq!(err.code, VALIDATION);
    for refused in [
        elsewhere.path().join("private.mp3"),
        work.join("missing.mp3"),
        Path::new("cuts/clip.mp4").to_path_buf(),
    ] {
        let err = app
            .call("files.media", link(&refused))
            .await
            .expect_err(&format!("{} is refused", refused.display()));
        assert_eq!(err.code, NOT_FOUND, "{}", refused.display());
    }

    let token = url.rsplit('/').next().expect("token");
    let wrong = url.replace(token, &"0".repeat(token.len()));
    assert_eq!(get(&wrong, None).await.status(), StatusCode::NOT_FOUND);

    // A file that is gone stops playing.
    fs::remove_file(&clip).expect("remove");
    assert_eq!(get(&url, None).await.status(), StatusCode::NOT_FOUND);
}
