//! Each bot's share of the weekly plan (spec 8.7), end to end: the turns'
//! cost from Claude Code's process total, the plan's rise while they ran,
//! and the share that comes out.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::bots::{Crew, two_bots};
use common::stream;
use serde_json::{Value, json};

const RESETS: i64 = 1_800_000_000;

fn week(utilization: f64) -> Value {
    json!({
        "type": "rate_limit_event",
        "rate_limit_info": { "status": "allowed", "resetsAt": RESETS,
            "unifiedWindows": { "seven_day": { "utilization": utilization, "resetsAt": RESETS } } },
    })
}

/// One turn of the lead: the plan reads `utilization` as it starts, and
/// the process has cost `total` by its end.
async fn turn(c: &mut Crew, lead: &Value, utilization: f64, total: f64) {
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
    c.lead_process.emit(week(utilization)).await;
    // The reading is in before the turn goes on.
    for _ in 0..500 {
        let read = c.t.daemon.usage().map(|usage| usage.windows[0].utilization);
        if read == Some(utilization) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    c.lead_process.emit(stream::init("session-1")).await;
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process.emit(stream::text("Done.")).await;
    c.t.clock.advance(Duration::from_secs(60));
    let mut result = stream::result(false);
    result["total_cost_usd"] = json!(total);
    c.lead_process.emit(result).await;
    c.t.until_state(lead, BotState::Idle).await;
}

#[tokio::test]
async fn the_share_is_learned_from_the_plans_rise_and_each_turns_cost() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();

    turn(&mut c, &lead, 0.30, 1.0).await;
    let early = c
        .app
        .call("usage.tokens", json!({ "since": 0 }))
        .await
        .expect("usage");
    assert_eq!(early[0]["planShare"], Value::Null, "nothing learned yet");

    // The week rose 3 points over the first turn, which cost $1.
    turn(&mut c, &lead, 0.33, 1.5).await;
    let usage = c
        .app
        .call("usage.tokens", json!({ "since": 0 }))
        .await
        .expect("usage");
    let share = usage[0]["planShare"].as_f64().expect("share");
    assert!(
        (share - 0.045).abs() < 1e-9,
        "{share}: $1.50 at 3 points a dollar"
    );

    let roster = c
        .writer
        .tool("crew_roster", json!({}))
        .await
        .expect("roster");
    assert_eq!(roster["bots"][0]["week_plan_percent"], 4.5);
}
