//! The history of a bot's memory and instructions (spec 8.10): each version
//! as it was, who made it, and a way back to it. The history lives in the
//! daemon's database, out of the bot's folder, so the bot cannot rewrite it.
//! With the params of the `versions.*` methods.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, VersionId};

text_enum!(
    /// What changed.
    VersionKind, "version kind" {
        /// The `CLAUDE.md` in the bot's folder, which the bot edits (5.1).
        Memory => "memory",
        /// The bot's instructions, which the owner, the bot (with the
        /// owner's OK) and the Maker change.
        Instructions => "instructions",
    }
);

text_enum!(
    /// Who made a version.
    VersionAuthor, "version author" {
        /// How it was when Botloft first looked: the start of the history.
        Start => "start",
        /// The owner, in the app (an edit, or going back to a version).
        Owner => "owner",
        /// The bot itself: its memory during its turn, or its instructions
        /// with `change_bot` and the owner's OK.
        Agent => "agent",
        /// The Maker, with a build or its Undo (spec 36).
        Maker => "maker",
        /// Someone outside Botloft, between the bot's turns: an editor, a
        /// script, another program.
        Outside => "outside",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TextVersion {
    pub id: VersionId,
    pub bot_id: BotId,
    pub kind: VersionKind,
    pub author: VersionAuthor,
    /// The whole text as it was; empty for a memory file that was gone.
    pub text: String,
    /// The version the owner went back to, when this one did that.
    pub restored_from: Option<VersionId>,
    /// Unix time in milliseconds.
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VersionsListParams {
    pub bot_id: BotId,
    pub kind: VersionKind,
}

/// Puts the bot's memory or instructions back as `version_id` had them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct VersionsRestoreParams {
    pub version_id: VersionId,
}
