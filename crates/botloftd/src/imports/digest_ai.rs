//! The summary of a claude.ai export (spec 36.6, 36.7), in markdown: the
//! owner's messages that repeat across conversations, joined as in 36.5,
//! then each conversation, newest first, with its title, date, how many
//! messages it has and the owner's first three messages. Up to
//! `CONVERSATIONS_MAX` conversations of the chosen period; past
//! `IMPORT_MAX_CHARS`, the oldest leave, and it says how many.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use botloft_core::protocol::IMPORT_MAX_CHARS;
use jiff::tz::TimeZone;

use super::claude_ai::{Conversation, Export};
use super::digest::Digest;
use super::group::{day, groups, when};
use super::redact::Redactor;
use super::words::Words;

/// The most conversations a summary takes, newest first.
pub const CONVERSATIONS_MAX: usize = 400;
/// Repeated messages listed.
const REPEATED_MAX: usize = 60;

/// Whether `conversation` started within the period.
fn within(conversation: &Conversation, from: Option<i64>, to: Option<i64>) -> bool {
    from.is_none_or(|from| conversation.started_at >= from)
        && to.is_none_or(|to| conversation.started_at <= to)
}

/// The summary of `export`, only the conversations started between `from`
/// and `to` when given.
pub fn build(
    export: &Export,
    from: Option<i64>,
    to: Option<i64>,
    redactor: &Redactor,
    words: &Words,
    tz: &TimeZone,
) -> Digest {
    let mut redactions = BTreeMap::new();
    let mut clean = |text: &str| redactor.clean(text, &mut redactions);
    let chosen: Vec<Conversation> = export
        .conversations
        .iter()
        .filter(|conversation| within(conversation, from, to))
        .take(CONVERSATIONS_MAX)
        .map(|conversation| Conversation {
            title: clean(&conversation.title),
            first: conversation.first.iter().map(|text| clean(text)).collect(),
            ..conversation.clone()
        })
        .collect();
    let first_at = chosen.last().map(|conversation| conversation.started_at);
    let last_at = chosen.first().map(|conversation| conversation.started_at);

    let mut text = String::new();
    let _ = writeln!(text, "# {}\n\n{}\n", words.title_ai, words.intro_ai);
    let _ = writeln!(
        text,
        "{}: {} – {} · {} {}\n",
        words.period,
        first_at.map(|at| day(at, tz)).unwrap_or_default(),
        last_at.map(|at| day(at, tz)).unwrap_or_default(),
        chosen.len(),
        words.count_ai,
    );
    if !export.projects.is_empty() {
        let _ = writeln!(text, "## {}\n", words.projects_ai);
        for project in &export.projects {
            let name = clean(project);
            let _ = writeln!(text, "- {name}");
        }
        text.push('\n');
    }
    let items = chosen.iter().flat_map(|conversation| {
        conversation
            .first
            .iter()
            .map(|text| (text.clone(), conversation.started_at))
    });
    let repeated: Vec<_> = groups(items, tz, usize::MAX)
        .into_iter()
        .filter(|group| group.count > 1)
        .take(REPEATED_MAX)
        .collect();
    if !repeated.is_empty() {
        let _ = writeln!(text, "## {}\n", words.repeated_ai);
        for group in &repeated {
            let _ = writeln!(text, "- {} — {}", group.text, when(group, words));
        }
        text.push('\n');
    }
    let _ = writeln!(text, "## {}\n", words.conversations);
    let mut cut = Vec::new();
    for conversation in &chosen {
        let mut section = String::new();
        let _ = writeln!(
            section,
            "### {}\n\n{} · {} {}\n",
            conversation.title,
            day(conversation.started_at, tz),
            conversation.messages,
            words.messages,
        );
        for message in &conversation.first {
            let _ = writeln!(section, "- {message}");
        }
        section.push('\n');
        if cut.is_empty()
            && text.chars().count() + section.chars().count() <= IMPORT_MAX_CHARS - 300
        {
            text.push_str(&section);
        } else {
            cut.push(conversation.title.clone());
        }
    }
    if !cut.is_empty() {
        let _ = writeln!(text, "{}: {}", words.cut_ai, cut.len());
    }
    Digest {
        text,
        scope: vec!["claude.ai".to_owned()],
        cut,
        redactions,
        from: first_at,
        to: last_at,
    }
}
