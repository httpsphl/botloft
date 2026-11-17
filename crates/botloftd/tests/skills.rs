//! The owner's skills in the agents (spec 38): which ones the owner has,
//! the copy an agent gets once given one, and an agent asking for one.
//! Skills for every agent and the menu of `/` are in `skills_everyone.rs`.

mod common;

use botloft_core::protocol::{BotState, SkillSource, SkillsShareParams};
use botloftd::skills;
use common::bots::two_bots;
use common::mcp::Mcp;
use common::skills::owner_skills;
use common::stream;
use common::supervised::setup;
use serde_json::{Value, json};

#[tokio::test(start_paused = true)]
async fn a_skill_given_to_an_agent_is_copied_into_its_folder_and_taken_back() {
    let s = setup().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    s.daemon.skills.use_root(root.path().to_path_buf());
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let workspace = first.spec.cwd.clone();

    let all = skills::list(&s.daemon).expect("list");
    let names: Vec<_> = all.skills.iter().map(|skill| skill.name.as_str()).collect();
    assert_eq!(names, ["deploy", "notes", "pdf"]);
    let deploy = &all.skills[0];
    assert_eq!(deploy.description, "Ships the app.");
    assert_eq!(deploy.allowed_tools, ["Bash(npm run build *)"]);
    assert!(deploy.has_files);
    assert_eq!(all.skills[2].source, SkillSource::ClaudeAi);

    let share = |bots: Vec<_>| SkillsShareParams {
        name: "deploy".into(),
        bot_ids: bots,
    };
    let shared = skills::share(&s.daemon, share(vec![s.bot.clone()])).expect("share");
    assert_eq!(shared.bot_ids, std::slice::from_ref(&s.bot));
    // It starts again with the copy in its folder and its name in the rules.
    s.runtime.process(2).await;
    let copy = workspace.join(".claude/skills/deploy");
    assert!(copy.join("SKILL.md").is_file());
    assert!(copy.join("scripts/ship.py").is_file());
    let rules = std::fs::read_to_string(workspace.join(".claude/rules/skills.md")).expect("rules");
    assert!(rules.contains("- deploy"), "{rules}");
    assert!(
        rules.contains("ask_skill") && rules.contains("- notes"),
        "{rules}"
    );

    // With the owner's folder gone, Botloft's own copy keeps it.
    std::fs::remove_dir_all(root.path().join("deploy")).expect("gone");
    let kept = skills::list(&s.daemon).expect("list");
    assert_eq!(kept.skills[0].name, "deploy");
    assert_eq!(kept.skills[0].source, SkillSource::Kept);

    skills::share(&s.daemon, share(Vec::new())).expect("taken back");
    s.runtime.process(3).await;
    assert!(!copy.exists());
}

#[tokio::test]
async fn an_agent_asks_for_a_skill_and_gets_it_when_the_owner_allows() {
    let mut c = two_bots().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    c.t.daemon.skills.use_root(root.path().to_path_buf());
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Go" }),
        )
        .await
        .expect("send");
    let lines = c.lead_process.wait_lines(1).await;
    c.lead_process
        .emit(stream::replay(lines.last().expect("line")))
        .await;

    let refused = c
        .lead
        .tool("ask_skill", json!({ "name": "unknown" }))
        .await
        .expect_err("not a skill");
    assert!(
        refused.contains("not one of the owner's skills"),
        "{refused}"
    );

    let mut asking = Mcp::new(c.lead.addr, c.lead.token.clone());
    let waiting = tokio::spawn(async move {
        asking
            .tool(
                "ask_skill",
                json!({ "name": "Deploy", "why": "To ship the site." }),
            )
            .await
    });
    let asked: Value = loop {
        let changed = c.app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            break body.clone();
        }
    };
    assert_eq!(asked["toolName"], "mcp__botloft__ask_skill");
    assert_eq!(asked["summary"], "deploy");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    assert_eq!(waiting.await.expect("task").expect("given")["done"], true);
    let listed = c
        .app
        .call("skills.list", json!(null))
        .await
        .expect("skills");
    assert_eq!(listed["skills"][0]["botIds"], json!([lead["id"]]));
}
