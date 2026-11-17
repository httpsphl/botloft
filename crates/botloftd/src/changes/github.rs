//! The pull requests the agent opened or touched, read from GitHub through
//! the owner's `gh` (spec 33.5): title, state, size and how the checks went.
//! The Botloft keeps no GitHub token: `gh` signs in as the owner did.

use std::io;
use std::path::Path;
use std::process::Output;
use std::sync::OnceLock;
use std::time::Duration;

use botloft_core::protocol::{
    GitEvent, GitEventKind, GithubAccess, PullChecks, PullRequest, PullState,
};
use serde_json::Value;

use crate::platform;

/// The longest one `gh` call may take.
const LIMIT: Duration = Duration::from_secs(10);
/// The most pull requests asked about, newest first.
const MAX_PRS: usize = 5;
const FIELDS: &str = "number,title,state,isDraft,additions,deletions,statusCheckRollup";
/// The program asked: the owner's `gh`, or what a test put in its place.
static PROGRAM: OnceLock<String> = OnceLock::new();
/// `gh` never stops to ask anything.
const ENV: [(&str, &str); 2] = [("GH_PROMPT_DISABLED", "1"), ("NO_COLOR", "1")];

fn program() -> &'static str {
    PROGRAM.get().map_or("gh", String::as_str)
}

/// Runs `program` instead of `gh`, for tests; the first call wins.
pub fn use_program(program: &str) {
    let _ = PROGRAM.set(program.to_owned());
}

/// The links of the pull requests in `events`, newest first, once each.
pub(super) fn links(events: &[GitEvent]) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    for event in events {
        if event.kind != GitEventKind::Pr {
            continue;
        }
        if let Some(url) = &event.url
            && !links.contains(url)
        {
            links.push(url.clone());
        }
        if links.len() == MAX_PRS {
            break;
        }
    }
    links
}

/// Asks GitHub about each link. Stops at the first answer that says `gh`
/// is missing or signed out: the others would say the same.
pub(super) fn read(links: &[String], cwd: &Path) -> (Vec<PullRequest>, Option<GithubAccess>) {
    if links.is_empty() {
        return (Vec::new(), None);
    }
    let mut prs = Vec::new();
    let mut access = GithubAccess::Ok;
    for url in links {
        let answer = platform::run_helper(
            program(),
            &["pr", "view", url, "--json", FIELDS],
            cwd,
            &ENV,
            LIMIT,
        );
        match outcome(answer) {
            Ok(json) => match pull_request(&json, url) {
                Some(pr) => prs.push(pr),
                None => access = GithubAccess::Error,
            },
            Err(GithubAccess::Missing) => return (prs, Some(GithubAccess::Missing)),
            Err(GithubAccess::SignedOut) => return (prs, Some(GithubAccess::SignedOut)),
            Err(other) => access = other,
        }
    }
    (prs, Some(access))
}

/// What one `gh` run says: its JSON, or why there is none.
fn outcome(answer: io::Result<Output>) -> Result<Value, GithubAccess> {
    let output = match answer {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(GithubAccess::Missing),
        Err(_) => return Err(GithubAccess::Error),
    };
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stderr).to_lowercase();
        // `gh` exits with 4 when it needs a sign-in, and says how.
        let signed_out = output.status.code() == Some(4)
            || said.contains("gh auth login")
            || said.contains("not logged");
        return Err(if signed_out {
            GithubAccess::SignedOut
        } else {
            GithubAccess::Error
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|_| GithubAccess::Error)
}

/// A pull request from `gh pr view --json`.
pub(super) fn pull_request(json: &Value, url: &str) -> Option<PullRequest> {
    let number = u32::try_from(json["number"].as_u64()?).ok()?;
    let state = match (json["state"].as_str()?, json["isDraft"].as_bool()) {
        ("MERGED", _) => PullState::Merged,
        ("CLOSED", _) => PullState::Closed,
        (_, Some(true)) => PullState::Draft,
        _ => PullState::Open,
    };
    let count = |field: &str| {
        json[field]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0)
    };
    Some(PullRequest {
        number,
        url: url.to_owned(),
        title: json["title"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect(),
        state,
        additions: count("additions"),
        deletions: count("deletions"),
        checks: checks(&json["statusCheckRollup"]),
    })
}

/// How the checks went: GitHub's check runs and the older status contexts.
fn checks(rollup: &Value) -> PullChecks {
    let mut checks = PullChecks {
        passed: 0,
        failed: 0,
        pending: 0,
    };
    for item in rollup.as_array().map_or(&[][..], Vec::as_slice) {
        let verdict = if item["__typename"] == "StatusContext" {
            item["state"].as_str().unwrap_or_default()
        } else if item["status"] == "COMPLETED" {
            item["conclusion"].as_str().unwrap_or_default()
        } else {
            "PENDING"
        };
        match verdict {
            "SUCCESS" | "NEUTRAL" | "SKIPPED" => checks.passed += 1,
            "FAILURE" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED" | "ERROR"
            | "STARTUP_FAILURE" => checks.failed += 1,
            _ => checks.pending += 1,
        }
    }
    checks
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// `gh pr view` of PR 301 of this repository on 2026-11-03, trimmed.
    fn answer() -> Value {
        json!({
            "number": 301, "state": "OPEN", "isDraft": false,
            "title": "feat(overview): an overview panel with what fills the conversation",
            "additions": 892, "deletions": 13,
            "statusCheckRollup": [
                { "__typename": "CheckRun", "name": "Rust (Windows)", "status": "COMPLETED",
                  "conclusion": "SUCCESS" },
                { "__typename": "CheckRun", "name": "Rust (Ubuntu)", "status": "COMPLETED",
                  "conclusion": "FAILURE" },
                { "__typename": "CheckRun", "name": "App (Windows)", "status": "IN_PROGRESS",
                  "conclusion": "" },
                { "__typename": "StatusContext", "context": "ci/legacy", "state": "SUCCESS" }
            ]
        })
    }

    #[test]
    fn a_pull_request_says_its_state_size_and_checks() {
        let pr = pull_request(&answer(), "https://github.com/httpsphl/botloft/pull/301")
            .expect("pull request");
        assert_eq!(pr.number, 301);
        assert_eq!(pr.state, PullState::Open);
        assert_eq!((pr.additions, pr.deletions), (892, 13));
        assert_eq!(
            (pr.checks.passed, pr.checks.failed, pr.checks.pending),
            (2, 1, 1)
        );
        let mut merged = answer();
        merged["state"] = json!("MERGED");
        let pr = pull_request(&merged, "u").expect("merged");
        assert_eq!(pr.state, PullState::Merged);
        let mut draft = answer();
        draft["isDraft"] = json!(true);
        assert_eq!(
            pull_request(&draft, "u").expect("draft").state,
            PullState::Draft
        );
    }

    #[test]
    fn links_are_the_newest_pull_requests_once_each() {
        let pr = |url: &str| GitEvent {
            kind: GitEventKind::Pr,
            sha: None,
            branch: None,
            action: None,
            number: Some(1),
            url: Some(url.into()),
            at: 0,
        };
        let events = [
            pr("a"),
            pr("b"),
            pr("a"),
            pr("c"),
            pr("d"),
            pr("e"),
            pr("f"),
        ];
        assert_eq!(links(&events), ["a", "b", "c", "d", "e"]);
        assert_eq!(read(&[], Path::new(".")), (Vec::new(), None));
    }

    #[test]
    fn a_missing_gh_and_a_sign_in_are_told_apart() {
        let missing = io::Error::new(io::ErrorKind::NotFound, "no gh");
        assert_eq!(outcome(Err(missing)), Err(GithubAccess::Missing));
    }
}
