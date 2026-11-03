//! What the owner set up for Claude Code themselves (spec 36.5): their
//! CLAUDE.md for every project and each project's own, and the name and
//! description (only the frontmatter) of their skills, commands, subagents
//! and workflows, for the owner and for each project. Nothing else of those
//! files is read into the summary.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read as _;
use std::path::Path;

use super::claude_code::{Project, cut};
use super::digest::RULES_MAX_CHARS;

/// The most of one file read, to find its frontmatter or CLAUDE.md.
const READ_MAX_BYTES: u64 = 64 * 1024;
/// How much of a description the summary keeps.
const DESCRIPTION_MAX_CHARS: usize = 200;

/// A skill, command, subagent or workflow, by its frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnTool {
    pub name: String,
    pub kind: &'static str,
    pub description: String,
}

/// What the summary adds besides the history itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extras {
    pub global_rules: Option<String>,
    pub global_tools: Vec<OwnTool>,
    /// By project key.
    pub project_rules: BTreeMap<String, String>,
    pub project_tools: BTreeMap<String, Vec<OwnTool>>,
}

fn read_start(path: &Path) -> Option<String> {
    let mut text = String::new();
    fs::File::open(path)
        .ok()?
        .take(READ_MAX_BYTES)
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

/// `name` and `description` from a file's frontmatter, if it has one.
fn frontmatter(text: &str) -> (Option<String>, Option<String>) {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (None, None);
    }
    let (mut name, mut description) = (None, None);
    for line in lines {
        let line = line.trim();
        if line == "---" {
            break;
        }
        let value = |prefix: &str| {
            line.strip_prefix(prefix)
                .map(|value| value.trim().trim_matches(['"', '\'']).to_owned())
                .filter(|value| !value.is_empty())
        };
        name = name.or_else(|| value("name:"));
        description = description.or_else(|| value("description:"));
    }
    (name, description)
}

fn tool(path: &Path, fallback: &str, kind: &'static str) -> Option<OwnTool> {
    let (name, description) = frontmatter(&read_start(path)?);
    Some(OwnTool {
        name: name.unwrap_or_else(|| fallback.to_owned()),
        kind,
        description: cut(&description.unwrap_or_default(), DESCRIPTION_MAX_CHARS),
    })
}

fn files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut found: Vec<_> = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The skills, commands, subagents and workflows set up in `dir` (a
/// `.claude` folder).
pub fn tools_in(dir: &Path) -> Vec<OwnTool> {
    let mut tools = Vec::new();
    for skill in files(&dir.join("skills"))
        .iter()
        .filter(|path| path.is_dir())
    {
        if let Some(found) = tool(&skill.join("SKILL.md"), &stem(skill), "skill") {
            tools.push(found);
        }
    }
    for (folder, kind) in [
        ("commands", "command"),
        ("agents", "subagent"),
        ("workflows", "workflow"),
    ] {
        for file in files(&dir.join(folder)) {
            let markdown = file.extension().is_some_and(|ext| ext == "md");
            if file.is_file()
                && markdown
                && let Some(found) = tool(&file, &stem(&file), kind)
            {
                tools.push(found);
            }
        }
    }
    tools
}

/// A project's CLAUDE.md, at its root or in its `.claude` folder.
fn project_rules(path: &Path) -> Option<String> {
    ["CLAUDE.md", ".claude/CLAUDE.md"]
        .iter()
        .find_map(|name| read_start(&path.join(name)))
        .map(|text| cut(&text, RULES_MAX_CHARS))
}

/// Everything the summary adds for `projects`, from Claude Code's folder
/// `root` and from the projects' own folders when they still exist.
pub fn gather(root: &Path, projects: &[Project]) -> Extras {
    let mut extras = Extras {
        global_rules: read_start(&root.join("CLAUDE.md")).map(|text| cut(&text, RULES_MAX_CHARS)),
        global_tools: tools_in(root),
        ..Extras::default()
    };
    for project in projects {
        let folder = Path::new(&project.path);
        if !folder.is_dir() {
            continue;
        }
        if let Some(rules) = project_rules(folder) {
            extras.project_rules.insert(project.key.clone(), rules);
        }
        let own = tools_in(&folder.join(".claude"));
        if !own.is_empty() {
            extras.project_tools.insert(project.key.clone(), own);
        }
    }
    extras
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_the_frontmatter() {
        let text = "---\nname: weekly-report\ndescription: \"Writes the weekly report\"\n---\nSecret body\n";
        assert_eq!(
            frontmatter(text),
            (
                Some("weekly-report".to_owned()),
                Some("Writes the weekly report".to_owned())
            )
        );
        assert_eq!(frontmatter("no frontmatter"), (None, None));
    }
}
