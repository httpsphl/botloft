//! The names Claude Code gives MCP servers in tool names (spec 34.5), and
//! the servers someone could have put in an agent's folder.

use std::path::Path;

use serde_json::Value;

/// How a server's name reads in its tools' names: what is not a letter, a
/// digit, `_` or `-` becomes `_` (`claude.ai Google Calendar` gives
/// `claude_ai_Google_Calendar`, as Claude Code 2.1.309 names it).
pub fn tool_server(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The tool-name server of a claude.ai connector, by its plain name.
pub fn connector_server(name: &str) -> String {
    tool_server(&format!("claude.ai {name}"))
}

/// The servers Claude Code would load for `workspace` besides the ones the
/// Botloft passes: its `.mcp.json`, and the local-scope ones `~/.claude.json`
/// keeps for that folder. A missing or unreadable file has none.
pub fn planted(workspace: &Path, claude_json: Option<&Path>) -> Vec<String> {
    let mut names = keys(&read(&workspace.join(".mcp.json"))["mcpServers"]);
    if let Some(path) = claude_json {
        let config = read(path);
        let wanted = folder_key(&workspace.to_string_lossy());
        if let Some(projects) = config["projects"].as_object() {
            for (folder, project) in projects {
                if folder_key(folder) == wanted {
                    names.extend(keys(&project["mcpServers"]));
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn read(path: &Path) -> Value {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or(Value::Null)
}

fn keys(servers: &Value) -> Vec<String> {
    servers
        .as_object()
        .map(|servers| servers.keys().cloned().collect())
        .unwrap_or_default()
}

/// A folder as `~/.claude.json` may write it: either slash, any case.
fn folder_key(folder: &str) -> String {
    folder
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn names_read_as_claude_code_writes_them_in_tools() {
        assert_eq!(
            connector_server("Google Calendar"),
            "claude_ai_Google_Calendar"
        );
        assert_eq!(
            connector_server("HyperFrames by HeyGen"),
            "claude_ai_HyperFrames_by_HeyGen"
        );
        assert_eq!(tool_server("my-crm v2"), "my-crm_v2");
    }

    #[test]
    fn servers_planted_in_the_folder_or_its_local_scope_are_found() {
        let home = tempfile::tempdir().expect("tempdir");
        let workspace = home.path().join("scout");
        std::fs::create_dir_all(&workspace).expect("folder");
        std::fs::write(
            workspace.join(".mcp.json"),
            json!({ "mcpServers": { "planted": { "command": "x" } } }).to_string(),
        )
        .expect("mcp.json");
        let claude_json = home.path().join(".claude.json");
        let key = workspace
            .to_string_lossy()
            .replace('\\', "/")
            .to_uppercase();
        std::fs::write(
            &claude_json,
            json!({ "projects": {
                key: { "mcpServers": { "local-one": { "command": "y" } } },
                "C:/elsewhere": { "mcpServers": { "other": { "command": "z" } } }
            } })
            .to_string(),
        )
        .expect("claude.json");
        assert_eq!(
            planted(&workspace, Some(&claude_json)),
            ["local-one", "planted"]
        );
        assert!(planted(&home.path().join("none"), None).is_empty());
    }
}
