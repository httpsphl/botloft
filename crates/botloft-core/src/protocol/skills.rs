//! The owner's skills in the agents (spec 38): the skills the owner has and
//! which agents may use each one.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

text_enum!(
    /// Where a skill comes from.
    SkillSource, "skill source" {
        /// The owner's own folder, `~/.claude/skills`.
        Personal => "personal",
        /// The owner's claude.ai account, kept in `~/.claude/skills/synced`.
        ClaudeAi => "claude_ai",
        /// Only Botloft's copy is left: the original is gone.
        Kept => "kept",
    }
);

/// A skill the owner can give to agents (spec 38.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Skill {
    /// Its folder's name, which is how Botloft knows it.
    pub name: String,
    /// What it does and when, from its `SKILL.md`; may be empty.
    pub description: String,
    pub source: SkillSource,
    /// Tools Claude Code runs without asking while the skill is used
    /// (`allowed-tools`).
    pub allowed_tools: Vec<String>,
    /// It runs commands by itself: hooks, or `` !`command` `` lines.
    pub runs_commands: bool,
    /// It brings other files than `SKILL.md`: scripts, references.
    pub has_files: bool,
    /// The agents that have it.
    pub bot_ids: Vec<BotId>,
}

/// What `skills.list` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SkillList {
    /// By name.
    pub skills: Vec<Skill>,
}

/// `skills.share`: the agents that may use a skill, in place of those
/// before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SkillsShareParams {
    pub name: String,
    pub bot_ids: Vec<BotId>,
}
