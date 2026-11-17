//! The summary the Maker reads (spec 36.5, 36.7), in markdown: for each
//! project, the requests the owner typed with the repeated ones joined (how
//! many times, on which days and at what hour: the best hint of a routine),
//! the session titles, the tools and models used and the project's
//! CLAUDE.md; first the owner's own CLAUDE.md and the names of their
//! skills, commands and subagents. Every text goes through the redactor.
//! Past `IMPORT_MAX_CHARS`, the oldest projects leave, and it says so.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use botloft_core::protocol::IMPORT_MAX_CHARS;
use jiff::tz::TimeZone;

use super::claude_code::{Project, cut};
use super::group::{Group, day, groups, when};
use super::own::{Extras, OwnTool};
use super::redact::Redactor;
use super::words::Words;

/// Repeated requests kept per project, newest first.
const GROUPS_MAX: usize = 150;
/// Session titles kept per project.
const TITLES_MAX: usize = 40;
/// Tools listed per project.
const TOOLS_MAX: usize = 20;
/// How much of a CLAUDE.md the summary keeps.
pub const RULES_MAX_CHARS: usize = 6_000;

/// The summary and what went into making it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    pub text: String,
    /// The projects in it, by path.
    pub scope: Vec<String>,
    /// The projects that left it for size, by path.
    pub cut: Vec<String>,
    pub redactions: BTreeMap<String, u32>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

struct Writer<'a> {
    redactor: &'a Redactor,
    words: &'a Words,
    tz: &'a TimeZone,
    redactions: BTreeMap<String, u32>,
}

impl Writer<'_> {
    fn clean(&mut self, text: &str) -> String {
        self.redactor.clean(text, &mut self.redactions)
    }

    fn groups(&mut self, project: &Project) -> Vec<Group> {
        let items: Vec<(String, i64)> = project
            .prompts
            .iter()
            .map(|prompt| (self.clean(&prompt.text), prompt.at))
            .collect();
        groups(items, self.tz, GROUPS_MAX)
    }

    fn own(&mut self, out: &mut String, tools: &[OwnTool]) {
        for tool in tools {
            let line = self.clean(&format!(
                "{} ({}): {}",
                tool.name, tool.kind, tool.description
            ));
            let _ = writeln!(out, "- {line}");
        }
    }

    fn rules(&mut self, out: &mut String, text: &str) {
        let text = self.clean(&cut(text.trim(), RULES_MAX_CHARS));
        let _ = writeln!(out, "~~~text\n{text}\n~~~\n");
    }

    fn project(&mut self, project: &Project, rules: Option<&str>, own: &[OwnTool]) -> String {
        let words = self.words;
        let mut out = String::new();
        let first = project.prompts.last().map(|prompt| day(prompt.at, self.tz));
        let last = project
            .prompts
            .first()
            .map(|prompt| day(prompt.at, self.tz));
        let path = self.clean(&project.path);
        let _ = writeln!(out, "## {path}\n");
        let _ = writeln!(
            out,
            "{} {} · {} {} · {} – {}\n",
            project.sessions,
            words.sessions,
            project.prompts.len(),
            words.prompts,
            first.unwrap_or_default(),
            last.unwrap_or_default(),
        );
        let _ = writeln!(out, "### {}\n", words.repeated);
        for group in self.groups(project) {
            if group.count > 1 {
                let when = when(&group, self.words);
                let _ = writeln!(out, "- {} — {when}", group.text);
            } else {
                let _ = writeln!(out, "- {}", group.text);
            }
        }
        out.push('\n');
        if !project.titles.is_empty() {
            let _ = writeln!(out, "### {}\n", words.titles);
            for title in project.titles.iter().take(TITLES_MAX) {
                let title = self.clean(&cut(title, 120));
                let _ = writeln!(out, "- {title}");
            }
            out.push('\n');
        }
        for (heading, counts, max) in [
            (words.tools, &project.tools, TOOLS_MAX),
            (words.models, &project.models, 6),
        ] {
            if counts.is_empty() {
                continue;
            }
            let mut sorted: Vec<(&String, &u32)> = counts.iter().collect();
            sorted.sort_by_key(|(name, count)| (std::cmp::Reverse(**count), (*name).clone()));
            let list: Vec<String> = sorted
                .into_iter()
                .take(max)
                .map(|(name, count)| format!("{name} {count}"))
                .collect();
            let _ = writeln!(out, "### {heading}\n\n{}\n", list.join(" · "));
        }
        if !own.is_empty() {
            let _ = writeln!(out, "### {}\n", words.own_tools);
            self.own(&mut out, own);
            out.push('\n');
        }
        if let Some(rules) = rules.filter(|rules| !rules.trim().is_empty()) {
            let _ = writeln!(out, "### {}\n", words.rules);
            self.rules(&mut out, rules);
        }
        out
    }
}

/// The summary of `projects` (newest first), with `extras`.
pub fn build(
    mut projects: Vec<Project>,
    extras: &Extras,
    redactor: &Redactor,
    words: &Words,
    tz: &TimeZone,
) -> Digest {
    projects.sort_by_key(|project| {
        std::cmp::Reverse(project.prompts.first().map_or(0, |prompt| prompt.at))
    });
    let mut writer = Writer {
        redactor,
        words,
        tz,
        redactions: BTreeMap::new(),
    };
    let mut head = String::new();
    let from = projects
        .iter()
        .filter_map(|p| p.prompts.last())
        .map(|p| p.at)
        .min();
    let to = projects
        .iter()
        .filter_map(|p| p.prompts.first())
        .map(|p| p.at)
        .max();
    let _ = writeln!(head, "# {}\n\n{}\n", words.title, words.intro);
    let _ = writeln!(
        head,
        "{}: {} – {}\n",
        words.period,
        from.map(|at| day(at, tz)).unwrap_or_default(),
        to.map(|at| day(at, tz)).unwrap_or_default()
    );
    if let Some(rules) = extras
        .global_rules
        .as_deref()
        .filter(|rules| !rules.trim().is_empty())
    {
        let _ = writeln!(head, "## {}\n", words.global_rules);
        writer.rules(&mut head, rules);
    }
    if !extras.global_tools.is_empty() {
        let _ = writeln!(head, "## {}\n", words.own_tools);
        writer.own(&mut head, &extras.global_tools);
        head.push('\n');
    }
    let mut text = head;
    let mut scope = Vec::new();
    let mut cut_paths = Vec::new();
    for project in &projects {
        let rules = extras.project_rules.get(&project.key).map(String::as_str);
        let own = extras
            .project_tools
            .get(&project.key)
            .map_or(&[][..], Vec::as_slice);
        let section = writer.project(project, rules, own);
        if text.chars().count() + section.chars().count() <= IMPORT_MAX_CHARS || scope.is_empty() {
            text.push_str(&section);
            scope.push(project.path.clone());
        } else {
            cut_paths.push(project.path.clone());
        }
    }
    if text.chars().count() > IMPORT_MAX_CHARS {
        text = format!(
            "{}\n\n{}\n",
            cut(&text, IMPORT_MAX_CHARS - 200),
            words.cut_short
        );
    }
    if !cut_paths.is_empty() {
        let _ = writeln!(text, "## {}\n", words.cut);
        for path in &cut_paths {
            let path = writer.clean(path);
            let _ = writeln!(text, "- {path}");
        }
    }
    Digest {
        text,
        scope,
        cut: cut_paths,
        redactions: writer.redactions,
        from,
        to,
    }
}
