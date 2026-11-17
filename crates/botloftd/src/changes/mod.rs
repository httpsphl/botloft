//! The Changes part of an agent's Overview (spec 33.5): what changed in the
//! git repositories its folders are in, and what it did in git, as Claude
//! Code reported each time a command of the agent touched git, with its
//! pull requests as GitHub has them now.

mod git;
mod github;

pub use github::use_program as use_github_program;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotIdParams, GitEvent, GitEventKind, OverviewChanges};
use serde_json::Value;
use tracing::{debug, warn};

use crate::service::bots::active;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// `overview.changes`: reads the repositories of the bot's folder and of its
/// crew's work folder, and lists what it did in git.
pub fn changes(daemon: &Daemon, params: BotIdParams) -> ApiResult<OverviewChanges> {
    let (folders, events) = {
        let store = daemon.store();
        let (crew, record) = active(&store, &params.bot_id)?;
        let folders = [
            daemon.paths.bot_workspace(&crew.slug, &record.slug),
            daemon.paths.work_folder(&crew),
        ];
        let events = store.git_events(&record.id).map_err(ApiError::from)?;
        (folders, events)
    };
    // The store is let go first: git can take a moment.
    let mut tops = Vec::new();
    for folder in &folders {
        if let Some(top) = git::top(folder)
            && !tops.contains(&top)
        {
            tops.push(top);
        }
    }
    let repos = tops.iter().filter_map(|top| git::read(top)).collect();
    // `gh` reads a full link from any folder; the bot's own is always there.
    let cwd = tops.first().unwrap_or(&folders[0]);
    let (prs, github) = github::read(&github::links(&events), cwd);
    Ok(OverviewChanges {
        repos,
        events,
        prs,
        github,
    })
}

/// The `gitOperation` of a command's result: each thing it did in git is
/// kept for the Overview.
pub(crate) fn operation(daemon: &Daemon, bot: &BotId, result: &Value) {
    let operation = &result["gitOperation"];
    if !operation.is_object() {
        return;
    }
    let at = daemon.clock.now_ms();
    for event in events(operation, at) {
        debug!(bot = %bot, kind = event.kind.as_str(), "git operation");
        if let Err(error) = daemon.store().add_git_event(bot, &event) {
            warn!(bot = %bot, %error, "could not keep a git operation");
        }
    }
}

fn text(value: &Value) -> Option<String> {
    value
        .as_str()
        .filter(|text| !text.is_empty())
        .map(|text| text.chars().take(200).collect())
}

/// The events of one `gitOperation`, in the order Claude Code lists them.
fn events(operation: &Value, at: i64) -> Vec<GitEvent> {
    let event = |kind| GitEvent {
        kind,
        sha: None,
        branch: None,
        action: None,
        number: None,
        url: None,
        at,
    };
    let mut events = Vec::new();
    let commit = &operation["commit"];
    if commit.is_object() {
        events.push(GitEvent {
            sha: text(&commit["sha"]).map(|sha| sha.chars().take(7).collect()),
            branch: text(&commit["branch"]),
            action: text(&commit["kind"]),
            ..event(GitEventKind::Commit)
        });
    }
    let push = &operation["push"];
    if push.is_object() {
        events.push(GitEvent {
            branch: text(&push["branch"]),
            ..event(GitEventKind::Push)
        });
    }
    let branch = &operation["branch"];
    if branch.is_object() {
        events.push(GitEvent {
            branch: text(&branch["ref"]),
            action: text(&branch["action"]),
            ..event(GitEventKind::Branch)
        });
    }
    let pr = &operation["pr"];
    if let Some(number) = pr["number"].as_u64().and_then(|n| u32::try_from(n).ok()) {
        events.push(GitEvent {
            number: Some(number),
            url: text(&pr["url"]).filter(|url| url.starts_with("https://")),
            action: text(&pr["action"]),
            ..event(GitEventKind::Pr)
        });
    }
    events
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_git_operation_becomes_one_event_per_thing_done() {
        // As the docs give `gitOperation` on a `Bash` result.
        let operation = json!({
            "commit": { "sha": "a1b2c3d4e5f6", "kind": "committed", "branch": "feat/site" },
            "push": { "branch": "feat/site" },
            "pr": { "number": 296, "url": "https://github.com/owner/site/pull/296",
                    "action": "created" }
        });
        let seen: Vec<_> = events(&operation, 7)
            .into_iter()
            .map(|event| {
                (
                    event.kind,
                    event.sha,
                    event.branch,
                    event.action,
                    event.number,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                (
                    GitEventKind::Commit,
                    Some("a1b2c3d".into()),
                    Some("feat/site".into()),
                    Some("committed".into()),
                    None
                ),
                (
                    GitEventKind::Push,
                    None,
                    Some("feat/site".into()),
                    None,
                    None
                ),
                (
                    GitEventKind::Pr,
                    None,
                    None,
                    Some("created".into()),
                    Some(296)
                ),
            ]
        );
    }

    #[test]
    fn a_link_that_is_not_https_is_left_out() {
        let operation =
            json!({ "pr": { "number": 1, "url": "javascript:alert(1)", "action": "created" } });
        let events = events(&operation, 0);
        assert_eq!(events[0].url, None);
        assert!(super::events(&json!({}), 0).is_empty());
    }
}
