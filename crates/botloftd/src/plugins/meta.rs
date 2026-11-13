//! What a plugin brings (spec 41.1), read from its files without running
//! anything: its manifest, skills, commands, subagents, hooks, language
//! servers, MCP servers and programs.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::found::{manifest, read_json};

/// What the owner sees of a plugin.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Meta {
    pub description: String,
    pub version: Option<String>,
    /// Skills and commands, as the agent calls them.
    pub skills: Vec<String>,
    pub agents: Vec<String>,
    /// The hook events its hooks run on.
    pub hooks: Vec<String>,
    pub language_servers: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub programs: bool,
    pub needs_settings: bool,
}

/// What the plugin in `dir`, called `name`, brings; nothing when the folder
/// is gone.
pub(super) fn read(dir: &Path, name: &str) -> Option<Meta> {
    if !dir.is_dir() {
        return None;
    }
    let manifest = manifest(dir);
    let text = |key: &str| manifest[key].as_str().map(str::to_owned);
    let mut skills = skills(dir, name, &manifest["skills"]);
    skills.extend(commands(dir, &manifest["commands"]));
    let mut meta = Meta {
        description: text("description").unwrap_or_default(),
        version: text("version"),
        skills: skills
            .into_iter()
            .map(|each| format!("{name}:{each}"))
            .collect(),
        agents: agents(dir, &manifest["agents"])
            .into_iter()
            .map(|each| format!("{name}:{each}"))
            .collect(),
        hooks: Vec::new(),
        language_servers: servers(dir, ".lsp.json", &manifest["lspServers"]),
        mcp_servers: servers(dir, ".mcp.json", &manifest["mcpServers"]),
        programs: dir
            .join("bin")
            .read_dir()
            .is_ok_and(|mut each| each.next().is_some()),
        needs_settings: manifest["userConfig"]
            .as_object()
            .is_some_and(|options| !options.is_empty()),
    };
    // `hooks/hooks.json` wraps its events in "hooks"; inline config does not.
    events(
        &read_json(&dir.join("hooks/hooks.json"))["hooks"],
        &mut meta.hooks,
    );
    for each in many(&manifest["hooks"]) {
        match each.as_str() {
            Some(path) => events(&read_json(&dir.join(path))["hooks"], &mut meta.hooks),
            None => events(each, &mut meta.hooks),
        }
    }
    for list in [&mut meta.skills, &mut meta.agents, &mut meta.hooks] {
        list.sort();
        list.dedup();
    }
    Some(meta)
}

/// A manifest key that takes one value or a list of them.
fn many(value: &Value) -> Vec<&Value> {
    match value {
        Value::Array(list) => list.iter().collect(),
        Value::Null => Vec::new(),
        one => vec![one],
    }
}

/// The paths a manifest key names, inside the plugin.
fn paths(dir: &Path, value: &Value) -> Vec<PathBuf> {
    many(value)
        .into_iter()
        .filter_map(Value::as_str)
        .filter(|path| !path.contains(".."))
        .map(|path| dir.join(path))
        .collect()
}

fn names_in(dir: &Path, wanted: impl Fn(&Path) -> Option<String>) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| wanted(&entry.path()))
        .collect()
}

fn stem(path: &Path) -> Option<String> {
    (path.extension()? == "md").then(|| {
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    })
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Its skills: each `<name>/SKILL.md` in `skills/` and in the folders the
/// manifest adds; a folder with a `SKILL.md` of its own is one skill. With
/// none of those, a `SKILL.md` at the root makes the plugin one skill.
fn skills(dir: &Path, name: &str, extra: &Value) -> Vec<String> {
    let default = dir.join("skills");
    let mut found = Vec::new();
    for folder in std::iter::once(default.clone()).chain(paths(dir, extra)) {
        if folder.join("SKILL.md").is_file() {
            let own = if folder == dir {
                name.to_owned()
            } else {
                folder_name(&folder)
            };
            found.push(own);
        } else {
            found.extend(names_in(&folder, |path| {
                path.join("SKILL.md").is_file().then(|| folder_name(path))
            }));
        }
    }
    if found.is_empty() && !default.is_dir() && extra.is_null() && dir.join("SKILL.md").is_file() {
        found.push(name.to_owned());
    }
    found
}

/// Its commands: the `.md` files in `commands/`, or what the manifest names
/// in its place (files, folders, or a map of names).
fn commands(dir: &Path, listed: &Value) -> Vec<String> {
    if let Some(map) = listed.as_object() {
        return map.keys().cloned().collect();
    }
    let places = if listed.is_null() {
        vec![dir.join("commands")]
    } else {
        paths(dir, listed)
    };
    let mut found = Vec::new();
    for place in places {
        if place.is_dir() {
            found.extend(names_in(&place, stem));
        } else if let Some(one) = stem(&place) {
            found.push(one);
        }
    }
    found
}

/// Its subagents: the `.md` files in `agents/`, or the files the manifest
/// names in its place.
fn agents(dir: &Path, listed: &Value) -> Vec<String> {
    if listed.is_null() {
        return names_in(&dir.join("agents"), stem);
    }
    paths(dir, listed)
        .iter()
        .filter_map(|path| stem(path))
        .collect()
}

/// The names of the servers in the plugin's own file (`.mcp.json`,
/// `.lsp.json`) and in what its manifest declares: a map of servers, a file
/// of them, or for MCP a bundle, named by its file.
fn servers(dir: &Path, file: &str, declared: &Value) -> Vec<String> {
    let mut found = Vec::new();
    take_servers(&read_json(&dir.join(file)), &mut found);
    for each in many(declared) {
        match each.as_str() {
            Some(path) if path.ends_with(".json") && !path.contains("..") => {
                take_servers(&read_json(&dir.join(path)), &mut found);
            }
            Some(path) => found.push(path.rsplit('/').next().unwrap_or(path).to_owned()),
            None => take_servers(each, &mut found),
        }
    }
    found.sort();
    found.dedup();
    found
}

/// The server names of a config: a map of servers, under `mcpServers` or
/// not.
fn take_servers(config: &Value, into: &mut Vec<String>) {
    let map = config.get("mcpServers").unwrap_or(config);
    if let Some(map) = map.as_object() {
        into.extend(map.keys().cloned());
    }
}

/// The hook events of a hooks config (`{"SessionStart": [...], ...}`).
fn events(config: &Value, into: &mut Vec<String>) {
    if let Some(map) = config.as_object() {
        into.extend(map.keys().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write(dir: &Path, path: &str, text: &str) {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, text).expect("write");
    }

    #[test]
    fn a_plugin_says_what_it_brings() {
        let dir = tempfile::tempdir().expect("dir");
        let d = dir.path();
        let manifest = json!({
            "name": "kit", "version": "0.5.0", "description": "Ships things.",
            "skills": ["./extra/"],
            "commands": ["./commands/deploy.md", "./more/"],
            "hooks": { "PostToolUse": [] },
            "mcpServers": { "api": { "command": "node" } },
            "lspServers": "./lsp.json",
            "userConfig": { "token": { "type": "string" } },
        });
        write(d, ".claude-plugin/plugin.json", &manifest.to_string());
        write(d, "skills/review/SKILL.md", "review");
        write(d, "skills/notes.txt", "not a skill");
        write(d, "extra/audit/SKILL.md", "audit");
        write(d, "commands/deploy.md", "deploy");
        write(d, "commands/ignored.md", "the manifest names its commands");
        write(d, "more/status.md", "status");
        write(d, "agents/helper.md", "helper");
        write(
            d,
            "hooks/hooks.json",
            r#"{"hooks":{"SessionStart":[],"PostToolUse":[]}}"#,
        );
        write(d, ".mcp.json", r#"{"mcpServers":{"docs":{"type":"http"}}}"#);
        write(d, "lsp.json", r#"{"go":{"command":"gopls"}}"#);
        write(d, "bin/tool.cmd", "@echo tool");

        let meta = read(d, "kit").expect("meta");
        assert_eq!(meta.description, "Ships things.");
        assert_eq!(meta.version.as_deref(), Some("0.5.0"));
        assert_eq!(
            meta.skills,
            ["kit:audit", "kit:deploy", "kit:review", "kit:status"]
        );
        assert_eq!(meta.agents, ["kit:helper"]);
        assert_eq!(meta.hooks, ["PostToolUse", "SessionStart"]);
        assert_eq!(meta.mcp_servers, ["api", "docs"]);
        assert_eq!(meta.language_servers, ["go"]);
        assert!(meta.programs);
        assert!(meta.needs_settings);
    }

    #[test]
    fn a_plugin_with_only_a_skill_at_its_root_is_one_skill() {
        let dir = tempfile::tempdir().expect("dir");
        write(dir.path(), "SKILL.md", "one");
        let meta = read(dir.path(), "solo").expect("meta");
        assert_eq!(meta.skills, ["solo:solo"]);
        assert!(meta.hooks.is_empty() && meta.mcp_servers.is_empty());
        assert!(!meta.programs && !meta.needs_settings);
        assert_eq!(read(&dir.path().join("gone"), "gone"), None);
    }
}
