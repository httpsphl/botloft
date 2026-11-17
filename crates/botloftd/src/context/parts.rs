//! What a bot's conversation holds, part by part (spec 33.3), from the
//! categories in Claude Code's answer to `get_context_usage`.

use std::fs;
use std::path::Path;

use botloft_core::protocol::{ContextFile, ContextPart, ContextPartKind};
use serde_json::Value;

/// The order the app lists the parts in.
const ORDER: [ContextPartKind; 9] = [
    ContextPartKind::Messages,
    ContextPartKind::Instructions,
    ContextPartKind::Tools,
    ContextPartKind::ConnectedTools,
    ContextPartKind::Memory,
    ContextPartKind::Skills,
    ContextPartKind::Agents,
    ContextPartKind::Other,
    ContextPartKind::Buffer,
];

/// The most memory files kept: a bot loads a handful.
const MAX_FILES: usize = 20;
/// The biggest memory file whose lines are counted: Claude Code skips a
/// bigger one.
const MAX_COUNTED: u64 = 4 * 1024 * 1024;

/// The categories that fill the conversation, grouped into the parts the app
/// names. Deferred tools and the free space are left out: they take no room.
pub(super) fn parts(body: &Value) -> Vec<ContextPart> {
    let mut parts: Vec<ContextPart> = Vec::new();
    for category in body["categories"].as_array().map_or(&[][..], Vec::as_slice) {
        let fills = matches!(category["kind"].as_str(), Some("used" | "buffer"));
        if !fills || category["isDeferred"].as_bool() == Some(true) {
            continue;
        }
        let Some(tokens) = category["tokens"].as_u64().filter(|tokens| *tokens > 0) else {
            continue;
        };
        let kind = kind_of(category);
        match parts.iter_mut().find(|part| part.kind == kind) {
            Some(part) => part.tokens += tokens,
            None => parts.push(ContextPart { kind, tokens }),
        }
    }
    parts.sort_by_key(|part| ORDER.iter().position(|kind| *kind == part.kind));
    parts
}

fn kind_of(category: &Value) -> ContextPartKind {
    match category["id"].as_str() {
        Some("messages") => ContextPartKind::Messages,
        Some("system_prompt") => ContextPartKind::Instructions,
        Some("system_tools") => ContextPartKind::Tools,
        Some("mcp_tools" | "mcp_instructions") => ContextPartKind::ConnectedTools,
        Some("memory_files") => ContextPartKind::Memory,
        Some("skills") => ContextPartKind::Skills,
        Some("custom_agents") => ContextPartKind::Agents,
        _ if category["kind"] == "buffer" => ContextPartKind::Buffer,
        _ => ContextPartKind::Other,
    }
}

/// The memory files the conversation holds, as Claude Code lists them,
/// with their lines: the agent's own memory should stay under 200
/// (spec 8.12).
pub(super) fn memory_files(body: &Value) -> Vec<ContextFile> {
    body["memoryFiles"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter_map(|file| {
            let path = file["path"].as_str().filter(|path| !path.is_empty())?;
            Some(ContextFile {
                path: path.to_owned(),
                tokens: file["tokens"].as_u64().unwrap_or(0),
                lines: lines_of(Path::new(path)),
            })
        })
        .take(MAX_FILES)
        .collect()
}

/// The lines of a memory file, counted as an editor does.
fn lines_of(path: &Path) -> Option<u32> {
    if fs::metadata(path).ok()?.len() > MAX_COUNTED {
        return None;
    }
    let text = fs::read(path).ok()?;
    u32::try_from(String::from_utf8_lossy(&text).lines().count()).ok()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// The answer of Claude Code 2.1.309 after one short turn, trimmed.
    fn answer() -> Value {
        json!({
            "categories": [
                {"id": "system_prompt", "name": "System prompt", "kind": "used", "tokens": 3341},
                {"id": "system_tools", "name": "System tools", "kind": "used", "tokens": 13908},
                {"id": "system_tools_deferred", "name": "System tools (deferred)",
                 "kind": "deferred", "isDeferred": true, "tokens": 13427},
                {"id": "mcp_tools", "name": "MCP tools", "kind": "used", "tokens": 2100},
                {"id": "mcp_instructions", "name": "MCP server instructions", "kind": "used", "tokens": 400},
                {"id": "memory_files", "name": "Memory files", "kind": "used", "tokens": 299},
                {"id": "skills", "name": "Skills", "kind": "used", "tokens": 2210},
                {"id": "messages", "name": "Messages", "kind": "used", "tokens": 4442},
                {"id": "something_new", "name": "Something new", "kind": "used", "tokens": 12},
                {"id": "autocompact_buffer", "name": "Autocompact buffer", "kind": "buffer", "tokens": 33000},
                {"id": "free_space", "name": "Free space", "kind": "free", "tokens": 942800}
            ],
            "memoryFiles": [
                {"path": "C:\\Users\\owner\\Botloft\\research\\scout\\CLAUDE.md", "type": "Project", "tokens": 299},
                {"path": "", "type": "Project", "tokens": 3}
            ]
        })
    }

    #[test]
    fn the_parts_group_the_categories_that_take_room() {
        let kinds: Vec<_> = parts(&answer())
            .into_iter()
            .map(|part| (part.kind, part.tokens))
            .collect();
        assert_eq!(
            kinds,
            [
                (ContextPartKind::Messages, 4442),
                (ContextPartKind::Instructions, 3341),
                (ContextPartKind::Tools, 13908),
                (ContextPartKind::ConnectedTools, 2500),
                (ContextPartKind::Memory, 299),
                (ContextPartKind::Skills, 2210),
                (ContextPartKind::Other, 12),
                (ContextPartKind::Buffer, 33000),
            ]
        );
    }

    #[test]
    fn an_answer_without_categories_has_no_parts() {
        assert!(parts(&json!({"totalTokens": 10})).is_empty());
        assert!(memory_files(&json!({})).is_empty());
    }

    #[test]
    fn memory_files_keep_their_path_and_size() {
        let files = memory_files(&answer());
        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with("CLAUDE.md"));
        assert_eq!(files[0].tokens, 299);
        // Not on this computer: no lines.
        assert_eq!(files[0].lines, None);
    }

    #[test]
    fn memory_files_say_how_many_lines_they_have() {
        let dir = tempfile::tempdir().expect("dir");
        let memory = dir.path().join("CLAUDE.md");
        fs::write(&memory, "# Memory\n\n- one\r\n- two\n").expect("memory");
        let answer = json!({"memoryFiles": [
            {"path": memory.to_string_lossy(), "tokens": 9},
            {"path": dir.path().join("gone.md").to_string_lossy(), "tokens": 1},
        ]});
        let lines: Vec<_> = memory_files(&answer)
            .iter()
            .map(|file| file.lines)
            .collect();
        assert_eq!(lines, [Some(4), None]);
    }
}
