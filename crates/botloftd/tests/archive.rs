//! `archive.list` (spec 7.6): what the owner archived, which the other
//! lists leave out, until it is deleted.

mod common;

use common::TestDaemon;
use serde_json::{Value, json};

async fn archive(app: &mut common::Client) -> Value {
    app.call("archive.list", json!(null))
        .await
        .expect("archive")
}

fn names(list: &Value) -> Vec<&str> {
    list.as_array()
        .expect("list")
        .iter()
        .map(|item| item["name"].as_str().expect("name"))
        .collect()
}

#[tokio::test]
async fn the_archive_lists_what_was_archived_until_it_is_deleted() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let docs = app
        .call("crews.create", json!({ "name": "Docs" }))
        .await
        .expect("crew");
    let mut bots = Vec::new();
    for (crew, name) in [(&ops, "Scout"), (&ops, "Writer"), (&docs, "Editor")] {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        bots.push(app.call("bots.create", bot).await.expect("bot"));
    }
    let empty = archive(&mut app).await;
    assert_eq!(empty, json!({ "crews": [], "bots": [] }));

    app.call("bots.archive", json!({ "botId": bots[0]["id"] }))
        .await
        .expect("archive a bot");
    app.call("crews.archive", json!({ "crewId": docs["id"] }))
        .await
        .expect("archive a crew");

    let listed = archive(&mut app).await;
    assert_eq!(names(&listed["crews"]), ["Docs"]);
    // The archived crew's bot comes too, the latest archived first.
    assert_eq!(names(&listed["bots"]), ["Editor", "Scout"]);
    let scout = &listed["bots"][1];
    assert_eq!(scout["state"], "archived");
    assert!(scout["archivedAt"].as_i64().is_some());
    let folder = t.paths.bot_workspace("ops", "scout");
    assert_eq!(scout["workspace"], folder.to_string_lossy().as_ref());
    let shared = t.paths.shared_dir("docs");
    assert_eq!(
        listed["crews"][0]["workFolder"],
        shared.to_string_lossy().as_ref()
    );

    app.call("crews.delete", json!({ "crewId": docs["id"] }))
        .await
        .expect("delete the crew");
    app.call("bots.delete", json!({ "botId": bots[0]["id"] }))
        .await
        .expect("delete the bot");
    assert_eq!(archive(&mut app).await, empty);
    // What was never archived is where it was.
    let active = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(names(&active), ["Writer"]);
}
