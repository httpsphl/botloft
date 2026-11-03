//! The Changes part of an agent's Overview (spec 33.5): the git repository
//! of its folder, and what it did in git as Claude Code reported it.

mod common;

use std::path::Path;
use std::process::Command;

use botloft_core::protocol::{BotIdParams, BotState, GitEventKind, GitFileStatus};
use botloftd::changes;
use common::context::{bot, settle};
use common::supervised::setup;
use serde_json::json;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com"])
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

#[tokio::test(start_paused = true)]
async fn the_agents_repository_shows_its_branch_and_changed_files() {
    let s = setup().await;
    let folder = bot(&s).workspace;
    let folder = Path::new(&folder);
    std::fs::create_dir_all(folder).expect("folder");
    git(folder, &["init", "-q", "-b", "main"]);
    std::fs::write(folder.join("report.md"), "one\ntwo\n").expect("write");
    git(folder, &["add", "report.md"]);
    git(folder, &["commit", "-q", "-m", "first"]);
    std::fs::write(folder.join("report.md"), "one\ntwo\nthree\n").expect("change");
    std::fs::write(folder.join("notes.txt"), "new").expect("new file");

    let found = changes::changes(
        &s.daemon,
        BotIdParams {
            bot_id: s.bot.clone(),
        },
    )
    .expect("changes");
    assert_eq!(found.repos.len(), 1, "the folder's repository, once");
    let repo = &found.repos[0];
    assert_eq!(repo.branch.as_deref(), Some("main"));
    let files: Vec<_> = repo
        .files
        .iter()
        .filter(|file| !file.path.starts_with('.'))
        .map(|file| (file.path.as_str(), file.status, file.added, file.removed))
        .collect();
    assert!(files.contains(&("report.md", GitFileStatus::Modified, Some(1), Some(0))));
    assert!(files.contains(&("notes.txt", GitFileStatus::Untracked, None, None)));
}

#[tokio::test(start_paused = true)]
async fn what_the_agent_did_in_git_is_kept_newest_first() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    // A `Bash` result as Claude Code gives it, with `gitOperation`.
    process
        .emit(json!({
            "type": "user", "parent_tool_use_id": null,
            "message": { "role": "user", "content": [
                { "type": "tool_result", "tool_use_id": "toolu_1", "content": "pushed" }] },
            "tool_use_result": { "stdout": "pushed", "stderr": "", "interrupted": false,
                "gitOperation": {
                    "push": { "branch": "feat/site" },
                    "pr": { "number": 12, "url": "https://github.com/owner/site/pull/12",
                            "action": "created" } } }
        }))
        .await;
    settle().await;

    let found = changes::changes(
        &s.daemon,
        BotIdParams {
            bot_id: s.bot.clone(),
        },
    )
    .expect("changes");
    let kinds: Vec<_> = found.events.iter().map(|event| event.kind).collect();
    assert_eq!(kinds, [GitEventKind::Pr, GitEventKind::Push]);
    assert_eq!(found.events[0].number, Some(12));
    assert_eq!(found.events[1].branch.as_deref(), Some("feat/site"));
}
