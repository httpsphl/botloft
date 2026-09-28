//! Files the owner sends to a bot (spec 9.5): saved in its folder, listed
//! in the prompt, images inline, and read back for the app.

mod common;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::bots::text_of;
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;

#[tokio::test]
async fn attachments_are_saved_and_images_go_inline() {
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
    let params = json!({ "botId": bot["id"], "body": "", "attachments": [
        { "name": "screen.png", "mediaType": "image/png", "data": BASE64.encode(b"png") },
        { "name": "../../report.pdf", "mediaType": "application/pdf", "data": BASE64.encode(b"pdf") },
    ] });
    let message = app
        .call("messages.send", params)
        .await
        .expect("files alone");
    let attachments = message["attachments"].as_array().expect("attachments");
    assert_eq!(attachments.len(), 2);
    assert_eq!(attachments[1]["name"], "report.pdf");
    let workspace = std::path::PathBuf::from(bot["workspace"].as_str().expect("workspace"));
    for attachment in attachments {
        let path = workspace.join(attachment["path"].as_str().expect("path"));
        assert!(path.is_file(), "{} missing", path.display());
    }

    let line = process.wait_lines(1).await.remove(0);
    let text = text_of(&line);
    assert!(
        text.starts_with("\n\nAttached files, saved in your folder: attachments/"),
        "{text}"
    );
    assert!(text.ends_with("/report.pdf"));
    assert_eq!(line["message"]["content"][1]["type"], "image");
    assert_eq!(
        line["message"]["content"][1]["source"]["data"],
        BASE64.encode(b"png")
    );

    // The app reads the files back to show them, while they are there.
    let read = |id: &Value| json!({ "attachmentId": id });
    let image = app
        .call("attachments.read", read(&attachments[0]["id"]))
        .await
        .expect("read");
    assert_eq!(
        image,
        json!({ "mediaType": "image/png", "data": BASE64.encode(b"png") })
    );
    std::fs::remove_file(workspace.join(attachments[1]["path"].as_str().expect("path")))
        .expect("remove");
    let gone = app
        .call("attachments.read", read(&attachments[1]["id"]))
        .await
        .expect_err("removed");
    assert_eq!(gone.code, NOT_FOUND);
}
