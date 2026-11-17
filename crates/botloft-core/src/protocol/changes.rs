//! What changed in an agent's git folders and what it did in git (spec
//! 33.5): the Changes part of the Overview panel.

use serde::{Deserialize, Serialize};

text_enum!(
    /// How a file differs from the last commit, from `git status`.
    GitFileStatus, "git file status" {
        Modified => "modified",
        Added => "added",
        Deleted => "deleted",
        Renamed => "renamed",
        /// Not yet known to git.
        Untracked => "untracked",
        /// In the middle of a merge that stopped on it.
        Conflicted => "conflicted",
    }
);

/// A changed file in a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GitFile {
    /// Relative to the repository, with `/`.
    pub path: String,
    pub status: GitFileStatus,
    /// Lines added and removed against the last commit; `null` for a new
    /// file, a binary one or a repository with no commit yet.
    pub added: Option<u32>,
    pub removed: Option<u32>,
}

/// A git repository one of the agent's folders is in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GitRepo {
    /// Absolute path of the repository's top folder.
    pub path: String,
    /// Its folder name.
    pub name: String,
    /// `null` when no branch is checked out.
    pub branch: Option<String>,
    /// Commits ahead of and behind its upstream; 0 without one.
    pub ahead: u32,
    pub behind: u32,
    /// The first changed files, at most 200.
    pub files: Vec<GitFile>,
    /// How many more changed files there are.
    pub more: u32,
}

text_enum!(
    /// What the agent did in git, from Claude Code's `gitOperation`.
    GitEventKind, "git event kind" {
        Commit => "commit",
        Push => "push",
        /// A branch merged or rebased.
        Branch => "branch",
        /// A pull request created, merged, closed...
        Pr => "pr",
    }
);

/// One thing the agent did in git (spec 33.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct GitEvent {
    pub kind: GitEventKind,
    /// A commit's short sha.
    pub sha: Option<String>,
    pub branch: Option<String>,
    /// As Claude Code says it: `committed`, `amended`, `merged`, `created`...
    pub action: Option<String>,
    /// A pull request's number and link.
    pub number: Option<u32>,
    pub url: Option<String>,
    /// Unix time in milliseconds.
    pub at: i64,
}

text_enum!(
    /// Where a pull request stands on GitHub.
    PullState, "pull request state" {
        Open => "open",
        Draft => "draft",
        Merged => "merged",
        Closed => "closed",
    }
);

/// How a pull request's checks went, counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PullChecks {
    pub passed: u32,
    pub failed: u32,
    /// Queued or running.
    pub pending: u32,
}

/// A pull request the agent opened or touched, as GitHub has it now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PullRequest {
    pub number: u32,
    pub url: String,
    pub title: String,
    pub state: PullState,
    pub additions: u32,
    pub deletions: u32,
    pub checks: PullChecks,
}

text_enum!(
    /// Whether the owner's `gh` could tell about the pull requests.
    GithubAccess, "github access" {
        Ok => "ok",
        /// `gh` is not installed.
        Missing => "missing",
        /// `gh` is installed but not signed in.
        SignedOut => "signed_out",
        Error => "error",
    }
);

/// What `overview.changes` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct OverviewChanges {
    pub repos: Vec<GitRepo>,
    /// Newest first, at most 50.
    pub events: Vec<GitEvent>,
    /// The newest pull requests of `events`, at most 5, as GitHub has them.
    pub prs: Vec<PullRequest>,
    /// `null` when there was no pull request to ask about.
    pub github: Option<GithubAccess>,
}
