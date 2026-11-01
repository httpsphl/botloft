//! Files from the phone (spec 28.14): sealed on the phone and left on the
//! server, taken and opened by the computer, attached the way the app does.

mod common;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use botloft_core::protocol::{FromPhone, PhoneFile, PhoneItem, ToPhone};
use botloftd::mobile::files::seal_file;
use common::Client;
use common::bots::ready_bot;
use common::cloud_server::CloudServer;
use common::mobile::{Phone, connect_phone, setup};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn bot_id(bot: &Value) -> botloft_core::ids::BotId {
    bot["id"].as_str().expect("id").parse().expect("bot id")
}

/// Seals `plain` with `key` and leaves it on the server, as the phone's page does.
async fn leave(cloud: &CloudServer, phone: &Phone, key: &[u8; 32], plain: &[u8]) -> String {
    let sealed = seal_file(key, plain).expect("seal");
    let response = reqwest::Client::new()
        .put(format!("{}/v1/files", cloud.url))
        .bearer_auth(&phone.token)
        .header("x-botloft-sha256", hex::encode(Sha256::digest(&sealed)))
        .header("content-length", sealed.len())
        .body(sealed)
        .send()
        .await
        .expect("put");
    assert_eq!(response.status(), 201);
    let answer: Value = response.json().await.expect("json");
    answer["id"].as_str().expect("id").to_owned()
}

fn file(id: &str, key: &[u8; 32], name: &str, size: usize) -> PhoneFile {
    PhoneFile {
        id: id.to_owned(),
        key: B64.encode(key),
        name: name.to_owned(),
        media_type: "text/plain".to_owned(),
        size: size as u64,
    }
}

async fn send(
    phone: &mut Phone,
    bot: &Value,
    id: &str,
    files: Vec<PhoneFile>,
) -> (bool, Option<String>) {
    phone
        .send(&FromPhone::Send {
            client_id: id.into(),
            bot_id: bot_id(bot),
            text: "Here it is".into(),
            files: Some(files),
        })
        .await;
    loop {
        if let ToPhone::Sent {
            client_id,
            ok,
            reason,
        } = phone.next().await
            && client_id == id
        {
            return (ok, reason);
        }
    }
}

async fn last_message(app: &mut Client, bot: &Value) -> Value {
    let listed = app
        .call("messages.list", json!({ "botId": bot["id"] }))
        .await
        .expect("list");
    listed[0].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_file_from_the_phone_reaches_the_bot_and_leaves_the_server() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Lead").await;
    drop(process);

    let key = [5u8; 32];
    let plain = b"total: 42.00";
    let id = leave(&cloud, &phone, &key, plain).await;
    let (ok, reason) = send(
        &mut phone,
        &bot,
        "f1",
        vec![file(&id, &key, "receipt.txt", plain.len())],
    )
    .await;
    assert!(ok, "{reason:?}");

    let message = last_message(&mut app, &bot).await;
    assert_eq!(message["body"], "Here it is");
    assert_eq!(message["attachments"][0]["name"], "receipt.txt");
    assert_eq!(message["attachments"][0]["size"], plain.len());
    let saved = walk(&t.paths.workspaces_root)
        .into_iter()
        .find(|path| path.ends_with("receipt.txt"))
        .expect("the file in the bot's folder");
    assert_eq!(std::fs::read(saved).expect("read"), plain);

    // The phone's own message lists the file by name.
    phone
        .send(&FromPhone::History {
            req: 1,
            bot_id: bot_id(&bot),
            before: None,
        })
        .await;
    let names = loop {
        if let ToPhone::History { items, .. } = phone.next().await
            && let Some(PhoneItem::You { files, .. }) = items
                .into_iter()
                .find(|item| matches!(item, PhoneItem::You { .. }))
        {
            break files;
        }
    };
    assert_eq!(names, Some(vec!["receipt.txt".to_owned()]));

    // The server forgot it: the same file again does not come down.
    let (again, why) = send(
        &mut phone,
        &bot,
        "f2",
        vec![file(&id, &key, "receipt.txt", plain.len())],
    )
    .await;
    assert!(!again);
    assert_eq!(why.as_deref(), Some("failed"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_file_that_does_not_open_or_is_not_the_servers_is_not_attached() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Lead").await;
    drop(process);

    let id = leave(&cloud, &phone, &[5u8; 32], b"secret").await;
    let (ok, reason) = send(
        &mut phone,
        &bot,
        "w1",
        vec![file(&id, &[6u8; 32], "a.txt", 6)],
    )
    .await;
    assert!(!ok);
    assert_eq!(reason.as_deref(), Some("failed"));

    for (n, bad) in [
        file("../copies", &[5u8; 32], "a.txt", 6),
        file(&id, &[5u8; 32], "a.txt", 0),
        PhoneFile {
            key: "short".into(),
            ..file(&id, &[5u8; 32], "a.txt", 6)
        },
    ]
    .into_iter()
    .enumerate()
    {
        let (ok, reason) = send(&mut phone, &bot, &format!("b{n}"), vec![bad]).await;
        assert!(!ok);
        assert_eq!(reason.as_deref(), Some("invalid"));
    }
    let listed = app
        .call("messages.list", json!({ "botId": bot["id"] }))
        .await
        .expect("list");
    assert_eq!(listed, json!([]));
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}
