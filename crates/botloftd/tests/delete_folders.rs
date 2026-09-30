//! Deleting a bot or a crew and sending its folder to the Recycle Bin
//! (spec 7.6). The test daemon's trash is a fake: folders move into a
//! folder of the test, never into a real bin.

mod common;

use std::io::ErrorKind;
use std::path::Path;

use common::{Client, TestDaemon};
use serde_json::{Value, json};

async fn crew_with_scout(app: &mut Client, work_folder: Option<&Path>) -> (Value, Value) {
    let mut crew = json!({ "name": "Ops" });
    if let Some(folder) = work_folder {
        crew["workFolder"] = json!(folder.to_string_lossy());
    }
    let crew = app.call("crews.create", crew).await.expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let scout = app.call("bots.create", bot).await.expect("bot");
    (crew, scout)
}

fn text(path: &Path) -> Value {
    json!(path.to_string_lossy())
}

#[tokio::test]
async fn a_deleted_bots_folder_goes_to_the_bin_when_asked() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let (_, scout) = crew_with_scout(&mut app, None).await;
    let workspace = t.paths.bot_workspace("ops", "scout");
    std::fs::write(workspace.join("CLAUDE.md"), "what the bot learned").expect("memory");

    app.call(
        "bots.delete",
        json!({ "botId": scout["id"], "recycleFolder": true }),
    )
    .await
    .expect("delete");
    let moved = app.notification("folder.recycled").await;
    assert_eq!(
        moved,
        json!({ "path": text(&workspace), "error": Value::Null })
    );
    assert!(!workspace.exists());
    assert_eq!(t.trash.moved(), [workspace]);
    // Only the bot's own folder went.
    assert!(t.paths.shared_dir("ops").is_dir());
}

#[tokio::test]
async fn without_asking_the_folder_stays_and_nothing_is_said() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let (crew, scout) = crew_with_scout(&mut app, None).await;
    for params in [
        json!({ "botId": scout["id"] }),
        json!({ "botId": scout["id"], "recycleFolder": false }),
    ] {
        // The second call finds no bot: the first one deleted it.
        let _ = app.call("bots.delete", params).await;
    }
    app.call("crews.delete", json!({ "crewId": crew["id"] }))
        .await
        .expect("delete the crew");
    assert_eq!(t.trash.tries(), 0);
    assert!(t.paths.bot_workspace("ops", "scout").is_dir());
    assert!(t.paths.shared_dir("ops").is_dir());
}

#[tokio::test]
async fn a_folder_still_in_use_is_tried_again() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let (_, scout) = crew_with_scout(&mut app, None).await;
    t.trash.fail_next(&[ErrorKind::PermissionDenied]);

    app.call(
        "bots.delete",
        json!({ "botId": scout["id"], "recycleFolder": true }),
    )
    .await
    .expect("delete");
    let moved = app.notification("folder.recycled").await;
    assert_eq!(moved["error"], Value::Null);
    assert_eq!(t.trash.tries(), 2);
    assert!(!t.paths.bot_workspace("ops", "scout").exists());
}

#[tokio::test]
async fn a_folder_the_bin_cannot_take_stays_and_the_app_is_told() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let (_, scout) = crew_with_scout(&mut app, None).await;
    t.trash.fail_next(&[ErrorKind::Unsupported]);
    let workspace = t.paths.bot_workspace("ops", "scout");

    // The bot is deleted all the same.
    app.call(
        "bots.delete",
        json!({ "botId": scout["id"], "recycleFolder": true }),
    )
    .await
    .expect("delete");
    let kept = app.notification("folder.recycled").await;
    assert_eq!(kept["path"], text(&workspace));
    assert_eq!(kept["error"], "fake failure");
    assert_eq!(t.trash.tries(), 1, "no use trying again");
    assert!(workspace.is_dir());
    assert_eq!(
        app.call("bots.list", json!({})).await.expect("bots"),
        json!([])
    );
}

#[tokio::test]
async fn a_deleted_crews_own_folder_goes_whole() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    // The owner's own folder, outside Botloft's: it is never moved.
    let project = t.paths.workspaces_root.with_file_name("project");
    let (crew, scout) = crew_with_scout(&mut app, Some(&project)).await;
    app.call("bots.archive", json!({ "botId": scout["id"] }))
        .await
        .expect("archive");
    std::fs::write(project.join("site.html"), "the owner's work").expect("work");
    let folder = t.paths.crew_dir("ops");

    app.call(
        "crews.delete",
        json!({ "crewId": crew["id"], "recycleFolder": true }),
    )
    .await
    .expect("delete");
    let moved = app.notification("folder.recycled").await;
    assert_eq!(
        moved,
        json!({ "path": text(&folder), "error": Value::Null })
    );
    assert!(!folder.exists(), "bots' folders and shared went with it");
    assert_eq!(
        std::fs::read_to_string(project.join("site.html")).expect("kept"),
        "the owner's work"
    );
}

#[tokio::test]
async fn a_folder_that_holds_the_owners_work_folder_stays() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    // The owner chose a work folder inside the crew's own folder.
    let inside = t.paths.crew_dir("ops").join("project");
    let (crew, _) = crew_with_scout(&mut app, Some(&inside)).await;
    let folder = t.paths.crew_dir("ops");

    app.call(
        "crews.delete",
        json!({ "crewId": crew["id"], "recycleFolder": true }),
    )
    .await
    .expect("delete");
    let kept = app.notification("folder.recycled").await;
    assert_eq!(kept["path"], text(&folder));
    assert_eq!(
        kept["error"],
        "the work folder you chose for the crew is inside it"
    );
    assert_eq!(t.trash.tries(), 0);
    assert!(inside.is_dir());
}
