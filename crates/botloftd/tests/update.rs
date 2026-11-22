//! `botloftd update` (spec 42.11) asks the daemon how many agents are in
//! the middle of a turn, as the owner, before it replaces anything.

mod common;

use botloft_core::protocol::BotState;
use common::{TOKEN, TestDaemon};
use serde_json::json;

#[tokio::test]
async fn an_update_counts_the_agents_in_a_turn() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let port = t.addr.port();
    assert_eq!(
        botloftd::update::busy_agents(port, TOKEN)
            .await
            .expect("asked"),
        0,
        "no agents"
    );

    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    t.until_state(&bot, BotState::Idle).await;
    let process = t.process_of(&bot).await;
    assert_eq!(
        botloftd::update::busy_agents(port, TOKEN)
            .await
            .expect("asked"),
        0,
        "an idle agent does not hold an update back"
    );

    let ask = json!({ "botId": bot["id"], "body": "Write the report" });
    app.call("messages.send", ask).await.expect("send");
    process.wait_lines(1).await;
    t.until_state(&bot, BotState::Busy).await;
    assert_eq!(
        botloftd::update::busy_agents(port, TOKEN)
            .await
            .expect("asked"),
        1
    );

    let refused = botloftd::update::busy_agents(port, "not-the-owner").await;
    assert!(refused.is_err(), "only the owner's token asks");
}
