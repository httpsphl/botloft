//! The design area (spec 22): a `Write` of an HTML file streamed by a fake
//! bot becomes a live draft at its `/view` address, gives way to the file
//! once the tool is done, and `/view` serves nothing outside the folders.

mod common;

use std::net::SocketAddr;
use std::path::PathBuf;

use common::bots::ready_bot;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct Page {
    status: u16,
    head: String,
    body: String,
}

/// GETs `url` (an `http://127.0.0.1:<port>/...` address) on `addr`.
async fn get(addr: SocketAddr, url: &str, origin: Option<&str>) -> Page {
    let path = url
        .split_once(&addr.to_string())
        .map_or(url, |(_, path)| path);
    let mut stream = TcpStream::connect(addr).await.expect("connect");
    let origin = origin.map_or(String::new(), |origin| format!("Origin: {origin}\r\n"));
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\n{origin}Connection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.expect("read");
    let text = String::from_utf8_lossy(&response).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").expect("http response");
    Page {
        status: head
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .expect("status"),
        head: head.to_lowercase(),
        body: body.to_owned(),
    }
}

fn stream_event(event: Value) -> Value {
    json!({ "type": "stream_event", "event": event, "parent_tool_use_id": null })
}

fn start(index: u64, id: &str, name: &str) -> Value {
    stream_event(json!({
        "type": "content_block_start", "index": index,
        "content_block": { "type": "tool_use", "id": id, "name": name, "input": {} },
    }))
}

fn piece(index: u64, json: &str) -> Value {
    stream_event(json!({
        "type": "content_block_delta", "index": index,
        "delta": { "type": "input_json_delta", "partial_json": json },
    }))
}

fn stop(index: u64) -> Value {
    stream_event(json!({ "type": "content_block_stop", "index": index }))
}

async fn draft(app: &mut Client) -> Value {
    app.notification("screen.draft").await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_screen_is_built_live_and_then_read_from_disk() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Site" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Designer").await;
    let crew = app.call("crews.list", Value::Null).await.expect("crews")[0].clone();
    let work = PathBuf::from(crew["workFolder"].as_str().expect("work folder"));
    let file = work.join("site").join("home.html");
    let path_json = serde_json::to_string(&file.to_string_lossy()).expect("path");

    process.emit(start(1, "toolu_home", "Write")).await;
    process
        .emit(piece(
            1,
            &format!("{{\"file_path\": {path_json}, \"content\": \"<h1>Bak"),
        ))
        .await;
    let first = draft(&mut app).await;
    assert_eq!(first["botId"], bot["id"]);
    assert_eq!(first["done"], false);
    assert_eq!(first["path"], file.to_string_lossy().as_ref());
    let url = first["url"].as_str().expect("url").to_owned();
    assert!(
        url.contains("/view/") && url.contains("/w/site/home.html?rev="),
        "{url}"
    );

    let page = get(t.addr, &url, None).await;
    assert_eq!(page.status, 200);
    assert_eq!(page.body, "<h1>Bak");
    assert!(
        page.head
            .contains("content-security-policy: sandbox allow-scripts")
    );
    assert!(page.head.contains("content-type: text/html; charset=utf-8"));
    assert!(page.head.contains("cache-control: no-store"));

    process
        .emit(piece(1, "ery</h1>\\n<p>Fresh \\\"bread\\\"</p>\"}"))
        .await;
    process.emit(stop(1)).await;
    let last = draft(&mut app).await;
    assert!(last["rev"].as_u64() > first["rev"].as_u64());
    let page = get(t.addr, last["url"].as_str().expect("url"), None).await;
    assert_eq!(page.body, "<h1>Bakery</h1>\n<p>Fresh \"bread\"</p>");

    // Not on disk yet: listed from the draft, being written.
    let screens = app
        .call("screens.list", json!({ "botId": bot["id"] }))
        .await
        .expect("screens");
    assert_eq!(screens[0]["name"], "home.html");
    assert_eq!(screens[0]["writing"], true);

    // Claude Code writes the file, and the tool result arrives.
    std::fs::create_dir_all(file.parent().expect("folder")).expect("folder");
    std::fs::write(
        &file,
        "<meta name=\"botloft-device\" content=\"mobile\"><h1>Bakery</h1><img src=\"logo.svg\">",
    )
    .expect("write");
    std::fs::write(work.join("site").join("logo.svg"), "<svg/>").expect("logo");
    process
        .emit(common::stream::tool_result(
            "toolu_home",
            "File created",
            false,
        ))
        .await;
    let done = draft(&mut app).await;
    assert_eq!(done["done"], true);
    let page = get(t.addr, &url, None).await;
    assert!(
        page.body.contains("<img src=\"logo.svg\">"),
        "{}",
        page.body
    );

    let screens = app
        .call("screens.list", json!({ "botId": bot["id"] }))
        .await
        .expect("screens");
    let listed = screens.as_array().expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["writing"], false);
    assert_eq!(listed[0]["device"], "mobile");
    assert_eq!(listed[0]["folder"], "site");
    let screen_url = listed[0]["url"].as_str().expect("url");
    assert!(screen_url.contains("?v="), "{screen_url}");

    // What is next to the page loads by the same address; the page itself,
    // in its sandbox, may fetch it.
    let logo = screen_url.replace("home.html", "logo.svg");
    let logo = logo.split('?').next().expect("address");
    let asset = get(t.addr, logo, Some("null")).await;
    assert_eq!(asset.status, 200);
    assert!(asset.head.contains("content-type: image/svg+xml"));
    assert!(asset.head.contains("access-control-allow-origin: null"));
}

#[tokio::test(flavor = "multi_thread")]
async fn view_serves_nothing_outside_the_bot_folders() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Site" }))
        .await
        .expect("crew");
    let (bot, _, _) = ready_bot(&t, &mut app, &crew, "Designer").await;
    let workspace = PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    std::fs::write(workspace.join("page.html"), "<p>mine</p>").expect("write");
    std::fs::create_dir_all(workspace.join(".hidden")).expect("hidden");
    std::fs::write(workspace.join(".hidden").join("x.html"), "no").expect("write");
    std::fs::create_dir_all(&t.paths.home).expect("home");
    std::fs::write(t.paths.home.join("secret.html"), "no").expect("write");

    let screens = app
        .call("screens.list", json!({ "botId": bot["id"] }))
        .await
        .expect("screens");
    let url = screens[0]["url"].as_str().expect("url").to_owned();
    assert!(url.contains("/b/page.html"), "{url}");
    assert_eq!(get(t.addr, &url, None).await.body, "<p>mine</p>");

    let base = url.split("/b/").next().expect("base").to_owned();
    for path in [
        "/b/.hidden/x.html",
        "/b/../../home/secret.html",
        "/b/..%2F..%2Fhome%2Fsecret.html",
        "/x/page.html",
        "/b/missing.html",
    ] {
        let page = get(t.addr, &format!("{base}{path}"), None).await;
        assert_eq!(page.status, 404, "{path}");
    }
    let wrong_key = url.replace("/view/", "/view/0");
    assert_eq!(get(t.addr, &wrong_key, None).await.status, 404);
}
