//! An agent's keys (spec 32): the owner saves one, the agent restarts with
//! it in its environment and its name in its rules, and the value is never
//! kept as it is in the chat, the messages or the rules.

mod common;

use botloft_core::protocol::{
    BotState, KeysDeleteParams, KeysListParams, KeysSetParams, MessagesSendParams,
};
use botloftd::service::{ApiError, keys, messages};
use common::stream;
use common::supervised::setup;

const TOKEN: &str = "wa-live-1234567890abcdef";

fn set(name: &str, value: &str) -> KeysSetParams {
    KeysSetParams {
        bot_id: botloft_core::ids::BotId::generate(),
        name: name.to_owned(),
        value: value.to_owned(),
        description: None,
    }
}

#[tokio::test(start_paused = true)]
async fn a_saved_key_reaches_the_agent_by_name_and_goes_when_deleted() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    assert_eq!(first.env("WA_TOKEN"), None);

    let listed = keys::set(
        &s.daemon,
        KeysSetParams {
            bot_id: s.bot.clone(),
            name: "WA_TOKEN".to_owned(),
            value: format!("  {TOKEN} "),
            description: Some("Token do gateway do WhatsApp".to_owned()),
        },
    )
    .expect("set");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "WA_TOKEN");
    assert!(!listed[0].missing);

    let second = s.runtime.process(2).await;
    assert!(first.killed(), "the agent restarts to get its key");
    assert_eq!(second.env("WA_TOKEN").as_deref(), Some(TOKEN));
    let rules =
        std::fs::read_to_string(second.spec.cwd.join(".claude/rules/keys.md")).expect("keys rules");
    assert!(
        rules.contains("`WA_TOKEN`: Token do gateway do WhatsApp"),
        "{rules}"
    );
    assert!(!rules.contains(TOKEN));
    let stored = keys::list(
        &s.daemon,
        KeysListParams {
            bot_id: s.bot.clone(),
        },
    )
    .expect("list");
    assert_eq!(stored, listed);

    s.until(BotState::Idle).await;
    keys::delete(
        &s.daemon,
        KeysDeleteParams {
            bot_id: s.bot.clone(),
            name: "WA_TOKEN".to_owned(),
        },
    )
    .expect("delete");
    let third = s.runtime.process(3).await;
    assert_eq!(third.env("WA_TOKEN"), None);
    assert!(!third.spec.cwd.join(".claude/rules/keys.md").exists());
}

#[tokio::test(start_paused = true)]
async fn what_the_agent_prints_and_the_owner_sends_keep_only_the_name() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let mut params = set("WA_TOKEN", TOKEN);
    params.bot_id = s.bot.clone();
    keys::set(&s.daemon, params).expect("set");
    let process = s.runtime.process(2).await;
    s.until(BotState::Idle).await;

    // The owner pastes the value in a message by mistake.
    let sent = messages::send(
        &s.daemon,
        MessagesSendParams {
            bot_id: s.bot.clone(),
            body: format!("use {TOKEN} please"),
            attachments: None,
            reply_to: None,
        },
    )
    .expect("send");
    assert_eq!(sent.body, "use [chave WA_TOKEN] please");

    // The agent echoes it.
    process
        .emit(stream::tool_use(
            "toolu_1",
            "PowerShell",
            serde_json::json!({ "command": format!("echo {TOKEN}") }),
        ))
        .await;
    process
        .emit(stream::tool_result("toolu_1", TOKEN, false))
        .await;
    process.emit(stream::text(&format!("It is {TOKEN}."))).await;
    let mut kept = String::new();
    for _ in 0..200 {
        let history = s
            .daemon
            .store()
            .chat_history(&s.bot, None, 20)
            .expect("history");
        kept = serde_json::to_string(&history).expect("json");
        if kept.contains("It is [chave WA_TOKEN].") {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(kept.contains("It is [chave WA_TOKEN]."), "{kept}");
    assert!(!kept.contains(TOKEN), "{kept}");
}

#[tokio::test(start_paused = true)]
async fn reserved_names_and_empty_values_are_refused() {
    let s = setup().await;
    s.runtime.process(1).await;
    for (name, value) in [
        ("PATH", TOKEN),
        ("ANTHROPIC_API_KEY", TOKEN),
        ("lower", TOKEN),
        ("WA_TOKEN", "  "),
    ] {
        let mut params = set(name, value);
        params.bot_id = s.bot.clone();
        assert!(
            matches!(keys::set(&s.daemon, params), Err(ApiError::Validation(_))),
            "{name}"
        );
    }
    assert!(matches!(
        keys::set(&s.daemon, set("WA_TOKEN", TOKEN)),
        Err(ApiError::NotFound(_))
    ));
}
