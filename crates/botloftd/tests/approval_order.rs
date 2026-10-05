//! A request for the owner is saved and waited on before the chat shows it
//! (spec 10.1): an answer sent the moment it shows always lands.

mod common;

use botloft_core::protocol::{ApprovalsAnswerParams, BotState, ChatBody};
use botloftd::state::Event;
use common::TestDaemon;
use common::mcp::Mcp;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn an_answer_the_moment_the_request_shows_lands() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let lead = json!({ "name": "Chefe", "role": "Leads", "instructions": "Build the site" });
    let crew = app
        .call("crews.create", json!({ "name": "Site", "lead": lead }))
        .await
        .expect("crew");
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let chief = bots[0].clone();
    let process = t.process_of(&chief).await;
    t.until_state(&chief, BotState::Idle).await;
    let mut mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));

    for round in 0..20 {
        let mut events = t.daemon.subscribe();
        let suggestion = json!({
            "name": format!("Designer {round}"),
            "role": "Draws each page",
            "instructions": "Design the pages.",
            "reason": "The site needs a look.",
        });
        let suggesting = tokio::spawn({
            let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
            async move { mcp.tool("suggest_bot", suggestion).await }
        });
        let approval_id = loop {
            if let Ok(Event::ChatItem(changed)) = events.recv().await
                && let ChatBody::Approval(item) = changed.item.body
            {
                break item.approval_id;
            }
        };
        // No wait at all between seeing it and answering.
        let answer: ApprovalsAnswerParams =
            serde_json::from_value(json!({ "approvalId": approval_id, "allow": false }))
                .expect("params");
        botloftd::approvals::answer(&t.daemon, answer).expect("the request is there");
        // And the bot hears it at once, not when the request runs out.
        let refused = tokio::time::timeout(std::time::Duration::from_secs(5), suggesting)
            .await
            .expect("the bot heard the answer")
            .expect("task")
            .expect("tool");
        assert_eq!(refused["created"], false, "{refused}");
    }
    mcp.tool("crew_roster", json!({}))
        .await
        .expect("still fine");
}
