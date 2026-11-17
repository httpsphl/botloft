//! A summary of the owner's history sent to the Maker, or forgotten
//! (spec 36.7). Reading the history itself is tested with a test `.claude`
//! folder in `imports::tests`; here the summary is made by hand, so no
//! test ever reads the developer's own history.

mod common;

use std::collections::BTreeMap;

use botloft_core::ids::ImportId;
use botloft_core::protocol::{Import, ImportSource};
use common::maker::maker;
use serde_json::json;

const SUMMARY: &str =
    "# Summary of my Claude Code history\n\n- Summarize today's e-mails — 14 times\n";

/// A summary waiting in `<home>\imports\`, as `imports.create` leaves it.
fn waiting(daemon: &botloftd::state::Daemon, created_at: i64, source: ImportSource) -> ImportId {
    let id = ImportId::generate();
    let dir = daemon.paths.home.join("imports");
    std::fs::create_dir_all(&dir).expect("imports folder");
    std::fs::write(dir.join(format!("{id}.md")), SUMMARY).expect("summary");
    daemon
        .store()
        .insert_import(&Import {
            id: id.clone(),
            source,
            created_at,
            sent_at: None,
            sent_message_id: None,
            scope: vec!["C:/work/blog".to_owned()],
            chars: SUMMARY.chars().count() as u32,
            redactions: BTreeMap::from([("key".to_owned(), 1)]),
        })
        .expect("import");
    id
}

#[tokio::test]
async fn a_summary_goes_to_the_maker_as_a_file_of_an_owners_message() {
    let mut m = maker().await;
    let now = m.t.daemon.clock.now_ms();
    let id = waiting(&m.t.daemon, now, ImportSource::ClaudeCode);
    let sent = m
        .app
        .call("imports.send", json!({ "importId": id.as_str() }))
        .await
        .expect("send");
    let messages = m
        .app
        .call("messages.list", json!({ "botId": m.bot["id"] }))
        .await
        .expect("messages");
    let message = messages
        .as_array()
        .expect("list")
        .iter()
        .find(|message| message["id"] == sent["messageId"])
        .expect("the owner's message")
        .clone();
    assert_eq!(message["fromKind"], "owner");
    assert_eq!(message["toBotId"], m.bot["id"]);
    assert_eq!(message["attachments"][0]["name"], "claude-code-history.md");
    let attachment = m
        .app
        .call(
            "attachments.read",
            json!({ "attachmentId": message["attachments"][0]["id"] }),
        )
        .await
        .expect("attachment");
    assert!(
        attachment["data"]
            .as_str()
            .is_some_and(|data| !data.is_empty())
    );

    // The file waiting in `imports` left, and the summary cannot go twice.
    let file =
        m.t.daemon
            .paths
            .home
            .join("imports")
            .join(format!("{id}.md"));
    assert!(!file.exists());
    let again = m
        .app
        .call("imports.send", json!({ "importId": id.as_str() }))
        .await;
    assert!(again.is_err());
    let list = m.app.call("imports.list", json!({})).await.expect("list");
    assert_eq!(list[0]["sentMessageId"], sent["messageId"]);
}

#[tokio::test]
async fn a_summary_is_forgotten_on_request_or_after_a_day() {
    let mut m = maker().await;
    let now = m.t.daemon.clock.now_ms();
    let kept = waiting(&m.t.daemon, now, ImportSource::ClaudeCode);
    m.app
        .call("imports.forget", json!({ "importId": kept.as_str() }))
        .await
        .expect("forget");
    let file = |id: &ImportId| {
        m.t.daemon
            .paths
            .home
            .join("imports")
            .join(format!("{id}.md"))
    };
    assert!(!file(&kept).exists());
    let gone = m
        .app
        .call("imports.send", json!({ "importId": kept.as_str() }))
        .await;
    assert!(gone.is_err());

    let old = waiting(
        &m.t.daemon,
        now - 25 * 60 * 60 * 1000,
        ImportSource::ClaudeCode,
    );
    let list = m.app.call("imports.list", json!({})).await.expect("list");
    assert_eq!(list.as_array().map(Vec::len), Some(0));
    assert!(!file(&old).exists());
}

#[tokio::test]
async fn a_claude_ai_summary_goes_as_its_own_file() {
    let mut m = maker().await;
    let now = m.t.daemon.clock.now_ms();
    let id = waiting(&m.t.daemon, now, ImportSource::ClaudeAi);
    let sent = m
        .app
        .call("imports.send", json!({ "importId": id.as_str() }))
        .await
        .expect("send");
    let messages = m
        .app
        .call("messages.list", json!({ "botId": m.bot["id"] }))
        .await
        .expect("messages");
    let message = messages
        .as_array()
        .expect("list")
        .iter()
        .find(|message| message["id"] == sent["messageId"])
        .expect("the owner's message")
        .clone();
    assert_eq!(message["attachments"][0]["name"], "claude-ai-history.md");
    assert!(
        message["body"]
            .as_str()
            .is_some_and(|body| body.contains("claude.ai"))
    );
}
