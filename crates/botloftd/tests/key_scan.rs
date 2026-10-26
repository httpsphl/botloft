//! Keys pasted before they were kept apart (spec 32.7): the scan shows only
//! their start and end, keeping one saves it for the agent and leaves its
//! name in the texts, and deleting one leaves "[chave removida]".

mod common;

use botloft_core::protocol::{BotState, KeyKeep, KeyScrub, KeysScrubParams, MessagesSendParams};
use botloftd::service::{ApiError, key_scan, messages};
use common::supervised::setup;

const GITHUB: &str = "ghp_abcdefghijklmnopqrstuvwxyz0123";
// In two parts, so the push protection of a code host does not take it for
// a real token.
const SLACK: &str = concat!("xox", "b-1234567890-abcdefghijklmnop");

#[tokio::test(start_paused = true)]
async fn pasted_keys_are_kept_by_name_or_deleted() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    messages::send(
        &s.daemon,
        MessagesSendParams {
            bot_id: s.bot.clone(),
            body: format!("the token is {GITHUB} and slack is {SLACK}"),
            attachments: None,
            reply_to: None,
        },
    )
    .expect("send");

    let hits = key_scan::scan(&s.daemon).expect("scan");
    assert_eq!(hits.len(), 2);
    let shown = serde_json::to_string(&hits).expect("json");
    assert!(!shown.contains(GITHUB) && !shown.contains(SLACK), "{shown}");
    let github = hits
        .iter()
        .find(|hit| hit.preview == "ghp_…0123")
        .expect("github");
    let slack = hits
        .iter()
        .find(|hit| hit.preview == "xoxb-…mnop")
        .expect("slack");
    assert_eq!(github.bot_ids, std::slice::from_ref(&s.bot));
    // The message and its chat item.
    assert_eq!(github.places, 2);

    // A refused name leaves everything as it was.
    let refused = key_scan::scrub(
        &s.daemon,
        KeysScrubParams {
            hits: vec![KeyScrub {
                id: github.id.clone(),
                keep: Some(KeyKeep {
                    bot_id: s.bot.clone(),
                    name: "PATH".to_owned(),
                }),
            }],
        },
    );
    assert!(matches!(refused, Err(ApiError::Validation(_))));
    assert_eq!(key_scan::scan(&s.daemon).expect("scan").len(), 2);

    let left = key_scan::scrub(
        &s.daemon,
        KeysScrubParams {
            hits: vec![
                KeyScrub {
                    id: github.id.clone(),
                    keep: Some(KeyKeep {
                        bot_id: s.bot.clone(),
                        name: "GITHUB_TOKEN".to_owned(),
                    }),
                },
                KeyScrub {
                    id: slack.id.clone(),
                    keep: None,
                },
            ],
        },
    )
    .expect("scrub");
    assert!(left.is_empty());

    let history = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 20)
        .expect("history");
    let kept = serde_json::to_string(&history).expect("json");
    assert!(
        kept.contains("the token is [chave GITHUB_TOKEN] and slack is [chave removida]"),
        "{kept}"
    );
    assert!(!kept.contains(GITHUB) && !kept.contains(SLACK), "{kept}");

    let second = s.runtime.process(2).await;
    assert!(first.killed(), "the agent restarts with the kept key");
    assert_eq!(second.env("GITHUB_TOKEN").as_deref(), Some(GITHUB));

    let gone = key_scan::scrub(
        &s.daemon,
        KeysScrubParams {
            hits: vec![KeyScrub {
                id: slack.id.clone(),
                keep: None,
            }],
        },
    );
    assert!(matches!(gone, Err(ApiError::NotFound(_))));
}
