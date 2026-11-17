//! What a skill says about itself (spec 38.1): the frontmatter of its
//! `SKILL.md`, read without a YAML library, and what the owner should know
//! before giving it to an agent.

use std::path::Path;

/// Longest `SKILL.md` read, in bytes.
const SKILL_MD_MAX: u64 = 512 * 1024;
/// Longest description kept, in characters, as Claude Code lists it.
const DESCRIPTION_MAX: usize = 1536;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Meta {
    pub description: String,
    pub allowed_tools: Vec<String>,
    pub runs_commands: bool,
    pub has_files: bool,
}

/// The skill in `dir`; `None` without a readable `SKILL.md`.
pub(super) fn read(dir: &Path) -> Option<Meta> {
    let file = dir.join("SKILL.md");
    if std::fs::metadata(&file).ok()?.len() > SKILL_MD_MAX {
        return None;
    }
    let text = std::fs::read_to_string(&file).ok()?;
    let mut meta = parse(&text);
    meta.has_files = std::fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .filter_map(Result::ok)
            .any(|entry| entry.file_name() != "SKILL.md")
    });
    Some(meta)
}

/// What the Markdown file at `file` says it does (`description` with
/// `when_to_use`): a skill's `SKILL.md` or a plugin's command. `None`
/// without the file or a description.
pub(super) fn description(file: &Path) -> Option<String> {
    if std::fs::metadata(file).ok()?.len() > SKILL_MD_MAX {
        return None;
    }
    let text = std::fs::read_to_string(file).ok()?;
    Some(parse(&text).description).filter(|description| !description.is_empty())
}

/// The frontmatter between the first two `---` lines, and the body after.
fn split(text: &str) -> (&str, &str) {
    let text = text.trim_start_matches('\u{feff}');
    let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    else {
        return ("", text);
    };
    match rest.find("\n---") {
        Some(end) => {
            let body = &rest[end + 4..];
            (
                &rest[..end],
                body.split_once('\n').map_or("", |(_, body)| body),
            )
        }
        None => ("", text),
    }
}

fn parse(text: &str) -> Meta {
    let (front, body) = split(text);
    let mut meta = Meta::default();
    let mut lines = front.lines().peekable();
    let mut extra = String::new();
    while let Some(line) = lines.next() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if line.starts_with([' ', '\t', '-']) {
            continue;
        }
        // A value on the next, indented lines: a block or a list.
        let mut value = value.trim().to_owned();
        let mut items = Vec::new();
        if value.is_empty() || value == "|" || value == ">" || value.starts_with(['|', '>']) {
            value.clear();
            while let Some(next) = lines.next_if(|next| next.starts_with([' ', '\t'])) {
                let next = next.trim();
                match next.strip_prefix("- ") {
                    Some(item) => items.push(unquote(item).to_owned()),
                    None if !next.is_empty() => {
                        if !value.is_empty() {
                            value.push(' ');
                        }
                        value.push_str(next);
                    }
                    None => {}
                }
            }
        }
        match key.trim() {
            "description" => meta.description = unquote(&value).to_owned(),
            "when_to_use" => extra = unquote(&value).to_owned(),
            "allowed-tools" if items.is_empty() => meta.allowed_tools = tools(unquote(&value)),
            "allowed-tools" => meta.allowed_tools = items,
            "hooks" => meta.runs_commands = true,
            _ => {}
        }
    }
    if !extra.is_empty() {
        if !meta.description.is_empty() {
            meta.description.push(' ');
        }
        meta.description.push_str(&extra);
    }
    if meta.description.chars().count() > DESCRIPTION_MAX {
        meta.description = meta.description.chars().take(DESCRIPTION_MAX).collect();
    }
    // A line that runs a command when the skill loads.
    meta.runs_commands |= body.contains("!`");
    meta
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|inner| inner.strip_suffix('\''))
        })
        .unwrap_or(value)
}

/// `Bash(git status *) Read, Grep` as its tools: spaces and commas split
/// them, except inside parentheses.
fn tools(value: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for ch in value.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ' ' | ',' if depth == 0 => {
                if !current.trim().is_empty() {
                    found.push(current.trim().to_owned());
                }
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        found.push(current.trim().to_owned());
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frontmatter_gives_what_the_owner_should_know() {
        let meta = parse(
            "---\nname: deploy\ndescription: >\n  Ships the app.\n  Carefully.\nwhen_to_use: \"On release\"\nallowed-tools: Bash(git status *) Bash(npm run build *), Read\n---\n\nRun !`git log -1` first.\n",
        );
        assert_eq!(meta.description, "Ships the app. Carefully. On release");
        assert_eq!(
            meta.allowed_tools,
            ["Bash(git status *)", "Bash(npm run build *)", "Read"]
        );
        assert!(meta.runs_commands);
    }

    #[test]
    fn a_list_of_tools_hooks_and_no_frontmatter_are_read_too() {
        let meta = parse(
            "---\ndescription: Reviews\nallowed-tools:\n  - Read\n  - Grep\nhooks:\n  PreToolUse: []\n---\nBody\n",
        );
        assert_eq!(meta.allowed_tools, ["Read", "Grep"]);
        assert!(meta.runs_commands);
        let plain = parse("Just instructions.\n");
        assert_eq!(plain, Meta::default());
    }
}
