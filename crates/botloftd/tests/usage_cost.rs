//! What the bots cost at API list prices (spec 8.7), end to end: each
//! turn's growth of Claude Code's running totals, per model, in the chat's
//! turn line, per bot, per model, per day and per routine, and in the
//! chief's roster.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotState, ChatBody, TurnItem};
use common::bots::{Crew, two_bots};
use common::stream;
use serde_json::{Value, json};

/// One turn of the lead whose process has cost `models` by its end.
async fn turn(c: &mut Crew, lead: &Value, models: Value) {
    c.t.clock.advance(Duration::from_secs(60));
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Go" }),
        )
        .await
        .expect("send");
    let line = c
        .lead_process
        .wait_lines(1)
        .await
        .last()
        .cloned()
        .expect("line");
    c.lead_process.emit(stream::init("session-1")).await;
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process.emit(stream::text("Done.")).await;
    let total: f64 = models
        .as_object()
        .expect("models")
        .values()
        .map(|cost| cost.as_f64().expect("cost"))
        .sum();
    let mut result = stream::result(false);
    result["total_cost_usd"] = json!(total);
    result["modelUsage"] = models
        .as_object()
        .expect("models")
        .iter()
        .map(|(model, cost)| (model.clone(), json!({ "costUSD": cost })))
        .collect();
    c.lead_process.emit(result).await;
    c.t.until_state(lead, BotState::Idle).await;
}

/// The lines that closed the lead's turns, oldest first.
fn turns(c: &Crew, lead: &Value) -> Vec<TurnItem> {
    let bot = lead["id"].as_str().expect("id").parse().expect("bot id");
    c.t.daemon
        .store()
        .chat_history(&bot, None, 100)
        .expect("history")
        .into_iter()
        .rev()
        .filter_map(|item| match item.body {
            ChatBody::Turn(turn) => Some(turn),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn each_turn_costs_what_the_running_totals_grew_by() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();

    turn(&mut c, &lead, json!({ "claude-opus-5-5": 1.0 })).await;
    // A subagent on Haiku joins; the main model grew by 30 cents.
    turn(
        &mut c,
        &lead,
        json!({ "claude-opus-5-5": 1.3, "claude-haiku-5-5": 0.2 }),
    )
    .await;

    let lines: Vec<_> = turns(&c, &lead)
        .iter()
        .map(|turn| turn.cost_micros)
        .collect();
    assert_eq!(lines, [Some(1_000_000), Some(500_000)]);

    let usage = c
        .app
        .call("usage.tokens", json!({ "since": 0 }))
        .await
        .expect("usage");
    assert_eq!(usage[0]["botId"], lead["id"]);
    assert!((usage[0]["cost"].as_f64().expect("cost") - 1.5).abs() < 1e-9);

    let models = c
        .app
        .call("usage.costs", json!({ "since": 0, "by": "model" }))
        .await
        .expect("models");
    assert_eq!(models[0]["key"], "claude-opus-5-5");
    assert!((models[0]["cost"].as_f64().expect("cost") - 1.3).abs() < 1e-9);
    assert_eq!(models[1]["key"], "claude-haiku-5-5");

    let days = c
        .app
        .call("usage.costs", json!({ "since": 0, "by": "day" }))
        .await
        .expect("days");
    assert_eq!(days.as_array().expect("days").len(), 1);

    let routines = c
        .app
        .call("usage.costs", json!({ "since": 0, "by": "routine" }))
        .await
        .expect("routines");
    assert_eq!(routines[0]["key"], "", "no routine carried these turns");
    assert!(routines[0].get("name").is_none());

    let roster = c
        .writer
        .tool("crew_roster", json!({}))
        .await
        .expect("roster");
    assert_eq!(roster["bots"][0]["week_cost_usd"], 1.5);

    let wrong = c
        .app
        .call("usage.costs", json!({ "since": -1, "by": "day" }))
        .await;
    assert!(wrong.is_err(), "since must not be negative");
}
