//! The owner's Claude Code plugins in the agents (spec 41): which ones are
//! on in the owner's Claude Code, the copy an agent loads with
//! `--plugin-dir` once given one, and plugins for every agent.

mod common;

use std::path::{Path, PathBuf};

use botloft_core::protocol::{BotState, PluginSource, PluginsShareParams};
use botloftd::plugins;
use common::bots::{ready_bot, two_bots};
use common::supervised::setup;
use serde_json::json;

fn write(dir: &Path, path: &str, text: &str) {
    let path = dir.join(path);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    std::fs::write(path, text).expect("write");
}

/// The owner's `.claude` folder: two plugins installed from a marketplace,
/// one of them turned off, and one from claude.ai. Answers where the
/// installed one that is on lives.
fn owner_plugins(claude: &Path) -> PathBuf {
    let cache = claude.join("plugins/cache/market");
    let vercel = cache.join("vercel/1.0.0");
    let manifest =
        json!({ "name": "vercel", "version": "1.0.0", "description": "Ships web apps." });
    write(&vercel, ".claude-plugin/plugin.json", &manifest.to_string());
    write(
        &vercel,
        "skills/deploy/SKILL.md",
        "---\ndescription: Deploys.\n---\nRead ${CLAUDE_PLUGIN_ROOT}/refs/steps.md\n",
    );
    write(&vercel, "refs/steps.md", "1. Build.\n");
    write(
        &vercel,
        "hooks/hooks.json",
        r#"{"hooks":{"SessionStart":[]}}"#,
    );
    write(
        &vercel,
        ".mcp.json",
        r#"{"mcpServers":{"vercel":{"type":"http"}}}"#,
    );
    let off = cache.join("off/1.0.0");
    write(&off, ".claude-plugin/plugin.json", r#"{"name":"off"}"#);
    let install = |dir: &Path| json!([{ "scope": "user", "installPath": dir }]);
    let record = json!({ "version": 2, "plugins": {
        "vercel@market": install(&vercel), "off@market": install(&off),
    }});
    write(
        claude,
        "plugins/installed_plugins.json",
        &record.to_string(),
    );
    let settings = json!({ "enabledPlugins": { "vercel@market": true, "off@market": false } });
    write(claude, "settings.json", &settings.to_string());
    let account = claude.join("plugins/synced/account-1");
    write(
        &account.join("data~g2"),
        ".claude-plugin/plugin.json",
        r#"{"name":"data"}"#,
    );
    write(&account.join("data~g2"), "skills/sql/SKILL.md", "SQL.\n");
    write(
        &account,
        "manifest.json",
        r#"{"plugins":[{"name":"data","generation":2}]}"#,
    );
    vercel
}

/// The folders a process was started with `--plugin-dir`.
fn plugin_dirs(args: &[String]) -> Vec<PathBuf> {
    args.iter()
        .enumerate()
        .filter(|(_, arg)| *arg == "--plugin-dir")
        .map(|(at, _)| PathBuf::from(&args[at + 1]))
        .collect()
}

#[tokio::test(start_paused = true)]
async fn a_plugin_given_to_an_agent_is_loaded_from_a_copy_in_its_folder_and_taken_back() {
    let s = setup().await;
    let claude = tempfile::tempdir().expect("claude");
    let vercel = owner_plugins(claude.path());
    s.daemon.plugins.use_root(claude.path().to_path_buf());
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let workspace = first.spec.cwd.clone();
    assert!(plugin_dirs(&first.args()).is_empty());

    let all = plugins::list(&s.daemon).expect("list");
    let names: Vec<_> = all
        .plugins
        .iter()
        .map(|plugin| plugin.name.as_str())
        .collect();
    assert_eq!(names, ["data", "vercel"]);
    let (data, listed) = (&all.plugins[0], &all.plugins[1]);
    assert_eq!(data.source, PluginSource::ClaudeAi);
    assert_eq!(data.skills, ["data:sql"]);
    assert_eq!(listed.source, PluginSource::ClaudeCode);
    assert_eq!(listed.version.as_deref(), Some("1.0.0"));
    assert_eq!(listed.description, "Ships web apps.");
    assert_eq!(listed.skills, ["vercel:deploy"]);
    assert_eq!(listed.hooks, ["SessionStart"]);
    assert_eq!(listed.mcp_servers, ["vercel"]);
    assert!(listed.bot_ids.is_empty());

    let share = |bots: Vec<_>| PluginsShareParams {
        name: "vercel".into(),
        bot_ids: bots,
    };
    let shared = plugins::share(&s.daemon, share(vec![s.bot.clone()])).expect("share");
    assert_eq!(shared.bot_ids, std::slice::from_ref(&s.bot));
    // It starts again loading the copy in its folder, without the servers.
    let second = s.runtime.process(2).await;
    let copy = workspace.join(".claude/plugins/vercel");
    assert_eq!(plugin_dirs(&second.args()), std::slice::from_ref(&copy));
    assert!(copy.join("skills/deploy/SKILL.md").is_file());
    assert!(copy.join("refs/steps.md").is_file());
    assert!(copy.join("hooks/hooks.json").is_file());
    assert!(!copy.join(".mcp.json").exists());

    // With the owner's plugin gone, Botloft's own copy keeps it.
    std::fs::remove_dir_all(&vercel).expect("gone");
    let kept = plugins::list(&s.daemon).expect("list");
    assert_eq!(kept.plugins[1].name, "vercel");
    assert_eq!(kept.plugins[1].source, PluginSource::Kept);
    assert_eq!(kept.plugins[1].skills, ["vercel:deploy"]);

    // Taken back, the copies go: no agent has it and the original is gone.
    plugins::share(&s.daemon, share(Vec::new())).expect("taken back");
    let third = s.runtime.process(3).await;
    assert!(plugin_dirs(&third.args()).is_empty());
    assert!(!copy.exists());
    assert!(!s.daemon.paths.home.join("plugins/vercel").exists());
    let left = plugins::list(&s.daemon).expect("list").plugins;
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].name, "data");

    let unknown = PluginsShareParams {
        name: "off".into(),
        bot_ids: vec![s.bot.clone()],
    };
    assert!(plugins::share(&s.daemon, unknown).is_err());
}

#[tokio::test]
async fn plugins_for_every_agent_reach_the_ones_made_later_and_come_back() {
    let mut c = two_bots().await;
    let claude = tempfile::tempdir().expect("claude");
    owner_plugins(claude.path());
    c.t.daemon.plugins.use_root(claude.path().to_path_buf());
    let bots = c.app.call("bots.list", json!({})).await.expect("bots");
    let lead = bots[0]["id"].clone();
    c.app
        .call(
            "plugins.share",
            json!({ "name": "vercel", "botIds": [lead] }),
        )
        .await
        .expect("share");
    c.app
        .call("plugins.everyone", json!({ "name": "data", "on": true }))
        .await
        .expect("data");
    // The writer starts again loading it.
    let writer = &bots[1];
    let old = c.writer_process.env("BOTLOFT_BOT_TOKEN");
    let fresh = loop {
        let process = c.t.process_of(writer).await;
        if process.env("BOTLOFT_BOT_TOKEN") != old {
            break process;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    };
    let loaded = plugin_dirs(&fresh.args());
    assert_eq!(loaded.len(), 1, "{loaded:?}");
    assert!(loaded[0].ends_with(".claude/plugins/data"), "{loaded:?}");

    let listed = c
        .app
        .call("plugins.everyone", json!({ "on": true }))
        .await
        .expect("all");
    assert_eq!(listed["everyone"], true);
    let (newcomer, _, _) = ready_bot(&c.t, &mut c.app, &c.crew, "Reviewer").await;
    let listed = c.app.call("plugins.list", json!(null)).await.expect("list");
    for plugin in listed["plugins"].as_array().expect("plugins") {
        let ids = plugin["botIds"].as_array().expect("ids");
        assert_eq!(ids.len(), 3, "{plugin}");
        assert!(ids.contains(&newcomer["id"]), "{plugin}");
    }
    let process = c.t.process_of(&newcomer).await;
    assert_eq!(plugin_dirs(&process.args()).len(), 2);

    // Taken back, each agent keeps what was chosen for it.
    let listed = c
        .app
        .call("plugins.everyone", json!({ "on": false }))
        .await
        .expect("off");
    let all = listed["plugins"].as_array().expect("plugins");
    assert_eq!(all[0]["name"], "data");
    assert_eq!(all[0]["everyone"], true);
    assert_eq!(all[0]["botIds"].as_array().expect("ids").len(), 3);
    assert_eq!(all[1]["name"], "vercel");
    assert_eq!(all[1]["botIds"], json!([lead]));
    assert!(
        c.app
            .call("plugins.everyone", json!({ "name": "unknown", "on": true }))
            .await
            .is_err()
    );
}
