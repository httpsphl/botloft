//! Skills for every agent (spec 38.3), a skill too big to copy, and the
//! menu of `/` in the chat (38.11).

mod common;

use botloft_core::protocol::{
    BotIdParams, BotState, SkillsEveryoneParams, SkillsShareParams, SlashSource,
};
use botloftd::skills;
use common::bots::{ready_bot, two_bots};
use common::mcp::Mcp;
use common::skills::owner_skills;
use common::supervised::setup;
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn skills_for_every_agent_reach_it_and_go_when_taken_back() {
    let s = setup().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    s.daemon.skills.use_root(root.path().to_path_buf());
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let skills_dir = first.spec.cwd.join(".claude/skills");
    let everyone = |name: Option<&str>, on: bool| SkillsEveryoneParams {
        name: name.map(Into::into),
        on,
    };

    // One skill for every agent.
    let listed = skills::everyone(&s.daemon, everyone(Some("notes"), true)).expect("notes");
    let notes = listed.skills.iter().find(|skill| skill.name == "notes");
    let notes = notes.expect("notes");
    assert!(notes.everyone && !listed.everyone);
    assert_eq!(notes.bot_ids, std::slice::from_ref(&s.bot));
    s.runtime.process(2).await;
    assert!(skills_dir.join("notes/SKILL.md").is_file());
    assert!(!skills_dir.join("deploy").exists());

    // Every skill, and one the owner adds later too.
    let listed = skills::everyone(&s.daemon, everyone(None, true)).expect("all");
    assert!(listed.everyone);
    assert!(
        listed
            .skills
            .iter()
            .all(|skill| skill.bot_ids == [s.bot.clone()])
    );
    s.runtime.process(3).await;
    assert!(skills_dir.join("deploy/scripts/ship.py").is_file());
    assert!(skills_dir.join("pdf/SKILL.md").is_file());
    let later = root.path().join("later");
    std::fs::create_dir_all(&later).expect("later");
    std::fs::write(later.join("SKILL.md"), "---\ndescription: Later.\n---\n").expect("skill");
    let listed = skills::list(&s.daemon).expect("list");
    let later = listed.skills.iter().find(|skill| skill.name == "later");
    assert_eq!(later.expect("later").bot_ids, std::slice::from_ref(&s.bot));

    // Taken back: the one still for every agent stays.
    skills::everyone(&s.daemon, everyone(None, false)).expect("all off");
    s.runtime.process(4).await;
    assert!(skills_dir.join("notes").is_dir());
    assert!(!skills_dir.join("deploy").exists());
    let rules = first.spec.cwd.join(".claude/rules/skills.md");
    let rules = std::fs::read_to_string(rules).expect("rules");
    let (has, others) = rules.split_once("ask_skill").expect("both parts");
    assert!(
        has.contains("- notes") && !has.contains("- deploy"),
        "{rules}"
    );
    assert!(
        others.contains("- deploy") && others.contains("- later"),
        "{rules}"
    );
    skills::everyone(&s.daemon, everyone(Some("notes"), false)).expect("notes off");
    s.runtime.process(5).await;
    assert!(!skills_dir.join("notes").exists());

    assert!(skills::everyone(&s.daemon, everyone(Some("unknown"), true)).is_err());
}

#[tokio::test]
async fn an_agent_made_later_gets_the_skills_for_every_agent() {
    let mut c = two_bots().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    c.t.daemon.skills.use_root(root.path().to_path_buf());
    let bots = c.app.call("bots.list", json!({})).await.expect("bots");
    let lead = bots[0]["id"].clone();
    c.app
        .call(
            "skills.share",
            json!({ "name": "deploy", "botIds": [lead] }),
        )
        .await
        .expect("share");
    c.app
        .call("skills.everyone", json!({ "name": "notes", "on": true }))
        .await
        .expect("notes");
    // The writer starts again with it, and asking for it is already done.
    let writer = &bots[1];
    let old = c.writer_process.env("BOTLOFT_BOT_TOKEN");
    let fresh = loop {
        let process = c.t.process_of(writer).await;
        if process.env("BOTLOFT_BOT_TOKEN") != old {
            break process;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    };
    c.t.until_state(writer, BotState::Idle).await;
    let token = fresh.env("BOTLOFT_BOT_TOKEN").expect("token");
    let have = Mcp::new(c.t.addr, token)
        .tool("ask_skill", json!({ "name": "notes" }))
        .await
        .expect("has it");
    assert_eq!(have["done"], true);

    let listed = c
        .app
        .call("skills.everyone", json!({ "on": true }))
        .await
        .expect("all");
    assert_eq!(listed["everyone"], true);
    let (newcomer, _, _) = ready_bot(&c.t, &mut c.app, &c.crew, "Reviewer").await;
    let listed = c.app.call("skills.list", json!(null)).await.expect("list");
    for skill in listed["skills"].as_array().expect("skills") {
        let ids = skill["botIds"].as_array().expect("ids");
        assert_eq!(ids.len(), 3, "{skill}");
        assert!(ids.contains(&newcomer["id"]), "{skill}");
    }

    // Taken back, each agent keeps what was chosen for it.
    let listed = c
        .app
        .call("skills.everyone", json!({ "on": false }))
        .await
        .expect("off");
    let skills = listed["skills"].as_array().expect("skills");
    let names: Vec<_> = skills.iter().map(|skill| skill["name"].clone()).collect();
    assert_eq!(names, [json!("deploy"), json!("notes"), json!("pdf")]);
    assert_eq!(skills[0]["botIds"], json!([lead]));
    assert_eq!(skills[1]["everyone"], true);
    assert_eq!(skills[1]["botIds"].as_array().expect("ids").len(), 3);
    assert_eq!(skills[2]["botIds"], json!([]));
    assert!(
        c.app
            .call("skills.everyone", json!({ "name": "unknown", "on": true }))
            .await
            .is_err()
    );
}

#[tokio::test(start_paused = true)]
async fn a_skill_too_big_to_copy_is_listed_but_reaches_no_agent() {
    let s = setup().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    let big = root.path().join("big");
    std::fs::create_dir_all(big.join("node_modules")).expect("big");
    std::fs::write(big.join("SKILL.md"), "---\ndescription: Too big.\n---\n").expect("skill");
    for n in 0..2001 {
        std::fs::write(big.join("node_modules").join(format!("{n}.js")), "").expect("file");
    }
    s.daemon.skills.use_root(root.path().to_path_buf());
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;

    let all = skills::everyone(
        &s.daemon,
        SkillsEveryoneParams {
            name: None,
            on: true,
        },
    )
    .expect("all");
    let listed = all
        .skills
        .iter()
        .find(|skill| skill.name == "big")
        .expect("listed");
    assert!(listed.too_big && listed.bot_ids.is_empty());
    s.runtime.process(2).await;
    let skills_dir = first.spec.cwd.join(".claude/skills");
    assert!(skills_dir.join("deploy").is_dir());
    assert!(!skills_dir.join("big").exists());
    let rules =
        std::fs::read_to_string(first.spec.cwd.join(".claude/rules/skills.md")).expect("rules");
    assert!(!rules.contains("- big"), "{rules}");

    let one = SkillsEveryoneParams {
        name: Some("big".into()),
        on: true,
    };
    let refused = skills::everyone(&s.daemon, one).expect_err("too big");
    assert!(refused.to_string().contains("too big"), "{refused}");
    let share = SkillsShareParams {
        name: "big".into(),
        bot_ids: vec![s.bot.clone()],
    };
    assert!(skills::share(&s.daemon, share).is_err());
}

#[tokio::test(start_paused = true)]
async fn the_menu_of_slash_lists_what_the_agent_can_run() {
    let s = setup().await;
    let root = tempfile::tempdir().expect("root");
    owner_skills(root.path());
    s.daemon.skills.use_root(root.path().to_path_buf());
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let share = SkillsShareParams {
        name: "deploy".into(),
        bot_ids: vec![s.bot.clone()],
    };
    skills::share(&s.daemon, share).expect("share");
    let second = s.runtime.process(2).await;
    // What its Claude Code lists: the skill it was given, one it was not,
    // one that comes with Claude Code, Claude Code's own commands and a
    // plugin's command.
    second
        .emit(json!({
            "type": "system", "subtype": "init", "session_id": "s", "tools": [],
            "skills": ["deploy", "notes", "code-review"],
            "slash_commands": ["deploy", "notes", "code-review", "model", "clear", "acme:ship"],
        }))
        .await;
    let params = || BotIdParams {
        bot_id: s.bot.clone(),
    };
    let mut listed = Vec::new();
    for _ in 0..200 {
        listed = skills::commands(&s.daemon, params())
            .expect("commands")
            .commands;
        if listed.len() > 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let names: Vec<_> = listed.iter().map(|command| command.name.as_str()).collect();
    assert_eq!(names, ["acme:ship", "code-review", "deploy"]);
    assert_eq!(listed[0].from, SlashSource::Plugin);
    assert_eq!(listed[1].from, SlashSource::ClaudeCode);
    assert_eq!(listed[2].from, SlashSource::Owner);
    assert_eq!(listed[2].description, "Ships the app.");

    // Taken back, it leaves the menu before Claude Code says so.
    let share = SkillsShareParams {
        name: "deploy".into(),
        bot_ids: Vec::new(),
    };
    skills::share(&s.daemon, share).expect("taken back");
    let after = skills::commands(&s.daemon, params())
        .expect("commands")
        .commands;
    assert!(after.iter().all(|command| command.name != "deploy"));
}
