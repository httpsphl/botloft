//! Where the owner's skills are (spec 38.1): the skills in the owner's own
//! Claude Code folder and the claude.ai ones kept there, and what each
//! agent's Claude Code listed, for the menu of `/` (38.11).

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use botloft_core::ids::BotId;
use botloft_core::protocol::SkillSource;

use crate::state::Daemon;

/// Where the owner's skills are looked for; the owner's own folder unless
/// a test says otherwise.
#[derive(Default)]
pub struct Skills {
    root: Mutex<Option<PathBuf>>,
    /// What each agent's Claude Code listed in its last `system/init`, for
    /// the menu of `/` (spec 38.11). Kept while the daemon runs.
    seen: Mutex<HashMap<BotId, Vec<String>>>,
}

impl Skills {
    /// Looks for the owner's skills in `root` instead of
    /// `~/.claude/skills`.
    pub fn use_root(&self, root: PathBuf) {
        *self.root.lock().unwrap_or_else(|p| p.into_inner()) = Some(root);
    }

    pub(super) fn root(&self) -> Option<PathBuf> {
        let set = self.root.lock().unwrap_or_else(|p| p.into_inner()).clone();
        set.or_else(|| std::env::home_dir().map(|home| home.join(".claude").join("skills")))
    }

    /// Keeps what the agent's Claude Code listed.
    pub(super) fn remember(&self, bot: &BotId, names: Vec<String>) {
        let mut seen = self.seen.lock().unwrap_or_else(|p| p.into_inner());
        seen.insert(bot.clone(), names);
    }

    /// What the agent's Claude Code listed last, if it started since the
    /// daemon did.
    pub(super) fn seen_by(&self, bot: &BotId) -> Vec<String> {
        let seen = self.seen.lock().unwrap_or_else(|p| p.into_inner());
        seen.get(bot).cloned().unwrap_or_default()
    }
}

/// A skill found in the owner's folders.
pub(super) struct Found {
    pub(super) dir: PathBuf,
    pub(super) source: SkillSource,
    /// Small enough for a copy (spec 38.3); a skill that is not goes to
    /// no agent.
    pub(super) fits: bool,
}

/// A folder name Botloft accepts as a skill's: what Claude Code would load,
/// and safe to join to a path.
pub(super) fn valid(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && !name.eq_ignore_ascii_case("synced")
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.' | ' '))
}

/// The skills in `dir`: each folder with a `SKILL.md`.
fn skills_in(dir: &Path, source: SkillSource, into: &mut BTreeMap<String, Found>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        // The folder itself may be a link the owner made; it is followed.
        if valid(&name) && path.join("SKILL.md").is_file() && !into.contains_key(&name) {
            let fits = super::copy::fits(&path);
            into.insert(
                name,
                Found {
                    dir: path,
                    source,
                    fits,
                },
            );
        }
    }
}

/// The owner's skills by name: their own win over the claude.ai ones.
pub(super) fn found(daemon: &Daemon) -> BTreeMap<String, Found> {
    let mut all = BTreeMap::new();
    let Some(root) = daemon.skills.root() else {
        return all;
    };
    skills_in(&root, SkillSource::Personal, &mut all);
    if let Ok(accounts) = std::fs::read_dir(root.join("synced")) {
        for account in accounts.filter_map(Result::ok) {
            skills_in(&account.path(), SkillSource::ClaudeAi, &mut all);
        }
    }
    all
}
