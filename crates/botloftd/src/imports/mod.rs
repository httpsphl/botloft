//! Bringing the owner's history to the Maker (spec 36.5, 36.6, 36.7):
//! reading Claude Code's history and the owner's export of claude.ai,
//! what the owner set up for Claude Code, the summary the owner reads
//! before it goes, and swapping out what must not go.

pub mod claude_ai;
pub mod claude_code;
pub mod digest;
pub mod digest_ai;
pub mod group;
pub mod own;
pub mod redact;
pub mod words;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_ai;
