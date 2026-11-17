//! The summary from a test `.claude` folder (spec 36.12, CR3): only the
//! owner's requests, no tool output nor answers, no project of Botloft's
//! own folders, repeated requests joined, keys swapped, the size limit.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use botloft_core::protocol::IMPORT_MAX_CHARS;
use jiff::tz::TimeZone;

use super::claude_code::{self, key_of};
use super::digest;
use super::own;
use super::redact::{Labels, Redactor};
use super::words::words;

fn user(cwd: &str, at: &str, content: serde_json::Value, extra: serde_json::Value) -> String {
    let mut record = serde_json::json!({
        "type": "user",
        "cwd": cwd,
        "timestamp": at,
        "sessionId": "s1",
        "message": { "role": "user", "content": content },
    });
    if let (Some(record), Some(extra)) = (record.as_object_mut(), extra.as_object()) {
        record.extend(extra.clone());
    }
    record.to_string()
}

fn session(root: &Path, cwd: &str, name: &str, lines: &[String]) {
    let dir = root.join("projects").join(key_of(cwd));
    fs::create_dir_all(&dir).expect("project folder");
    fs::write(dir.join(format!("{name}.jsonl")), lines.join("\n")).expect("session");
}

/// A `.claude` folder: a blog project, a project inside Botloft's own
/// folder, and a request kept only in `history.jsonl`.
fn claude_folder(root: &Path) {
    let blog = "C:\\work\\blog";
    let none = serde_json::json!({});
    session(
        root,
        blog,
        "s1",
        &[
            user(blog, "2026-11-02T12:00:00Z", "Resuma os e-mails de hoje".into(), none.clone()),
            user(blog, "2026-11-03T12:05:00Z", "resuma os e-mails de hoje!".into(), none.clone()),
            user(
                blog,
                "2026-11-03T12:10:00Z",
                serde_json::json!([{ "type": "tool_result", "tool_use_id": "t", "content": "TOOL-OUTPUT sk-ant-api03-zzzzzzzzzzzzzzzz" }]),
                none.clone(),
            ),
            user(blog, "2026-11-03T12:11:00Z", "Use a chave sk-ant-api03-abcdefghijklmnop no deploy".into(), none.clone()),
            user(blog, "2026-11-03T12:12:00Z", "<command-name>/clear</command-name>".into(), none.clone()),
            user(blog, "2026-11-03T12:13:00Z", "a peer asked this".into(), serde_json::json!({ "origin": { "kind": "peer" } })),
            user(blog, "2026-11-03T12:14:00Z", "an sdk prompt".into(), serde_json::json!({ "promptSource": "sdk" })),
            serde_json::json!({ "type": "assistant", "message": { "model": "claude-opus-5-5", "content": [
                { "type": "text", "text": "ASSISTANT-ANSWER" },
                { "type": "tool_use", "name": "Bash", "input": { "command": "TOOL-INPUT" } },
                { "type": "tool_use", "name": "mcp__notion__search", "input": {} }
            ] } })
            .to_string(),
            serde_json::json!({ "type": "custom-title", "customTitle": "Newsletter semanal", "sessionId": "s1" }).to_string(),
        ],
    );
    let agent = "C:\\Botloft\\workspaces\\ops\\scout";
    session(
        root,
        agent,
        "s2",
        &[user(
            agent,
            "2026-11-03T10:00:00Z",
            "AGENT-WORK".into(),
            none,
        )],
    );
    fs::write(
        root.join("history.jsonl"),
        [
            serde_json::json!({ "display": "Publique o post da semana", "pastedContents": {}, "project": "C:\\work\\old", "sessionId": "gone", "timestamp": 1_760_000_000_000_i64 }).to_string(),
            serde_json::json!({ "display": "/clear", "pastedContents": {}, "project": "C:\\work\\old", "sessionId": "gone", "timestamp": 1_760_000_100_000_i64 }).to_string(),
        ]
        .join("\n"),
    )
    .expect("history");
    fs::write(root.join("CLAUDE.md"), "Sempre responda em português.").expect("rules");
    fs::create_dir_all(root.join("commands")).expect("commands");
    fs::write(
        root.join("commands").join("relatorio.md"),
        "---\ndescription: Escreve o relatório semanal\n---\nCORPO-DO-COMANDO",
    )
    .expect("command");
}

fn skip() -> Vec<String> {
    vec!["C:\\Botloft".to_owned()]
}

#[test]
fn lists_the_owners_projects_without_botlofts_own() {
    let dir = tempfile::tempdir().expect("dir");
    claude_folder(dir.path());
    let projects = claude_code::read(dir.path(), None, false, &skip());
    let paths: Vec<&str> = projects.iter().map(|p| p.path.as_str()).collect();
    assert_eq!(paths, ["C:\\work\\blog", "C:\\work\\old"]);
    let blog = &projects[0];
    assert_eq!((blog.sessions, blog.prompts.len()), (1, 3));
    assert_eq!(projects[1].prompts.len(), 1);
}

#[test]
fn the_summary_has_only_what_the_owner_typed_with_keys_swapped() {
    let dir = tempfile::tempdir().expect("dir");
    claude_folder(dir.path());
    let keys: HashSet<String> = [key_of("C:\\work\\blog"), key_of("C:\\work\\old")].into();
    let projects = claude_code::read(dir.path(), Some(&keys), true, &skip());
    let extras = own::gather(dir.path(), &projects);
    let redactor = Redactor::new(Labels::for_locale("pt-BR"), Vec::new());
    let digest = digest::build(projects, &extras, &redactor, words("pt-BR"), &TimeZone::UTC);
    let text = &digest.text;
    for absent in [
        "TOOL-OUTPUT",
        "TOOL-INPUT",
        "ASSISTANT-ANSWER",
        "AGENT-WORK",
        "CORPO-DO-COMANDO",
        "sk-ant",
        "/clear",
        "a peer asked",
        "an sdk prompt",
    ] {
        assert!(!text.contains(absent), "{absent} leaked into:\n{text}");
    }
    // The two e-mail requests are one, made twice, in its newest wording.
    assert!(
        text.contains("- resuma os e-mails de hoje! — 2 vezes, seg/ter ~12h"),
        "{text}"
    );
    assert!(text.contains("Use a chave [chave] no deploy"), "{text}");
    assert!(text.contains("Newsletter semanal"), "{text}");
    assert!(text.contains("Bash 1 · mcp__notion 1"), "{text}");
    assert!(text.contains("claude-opus-5-5 1"), "{text}");
    assert!(text.contains("Publique o post da semana"), "{text}");
    assert!(text.contains("Sempre responda em português."), "{text}");
    assert!(
        text.contains("relatorio (command): Escreve o relatório semanal"),
        "{text}"
    );
    assert_eq!(digest.redactions.get("key"), Some(&1));
    assert_eq!(digest.scope, ["C:\\work\\blog", "C:\\work\\old"]);
}

#[test]
fn past_the_limit_the_oldest_projects_leave() {
    let dir = tempfile::tempdir().expect("dir");
    let none = serde_json::json!({});
    // Six projects of about 41 000 characters each, a day apart.
    for index in 0..6 {
        let cwd = format!("C:\\work\\p{index}");
        let lines: Vec<String> = (0..140)
            .map(|n| {
                let text = format!("pedido {n} {}", "x".repeat(280));
                let at = format!("2026-10-{:02}T10:{:02}:00Z", index + 1, n % 60);
                user(&cwd, &at, text.into(), none.clone())
            })
            .collect();
        session(dir.path(), &cwd, "s", &lines);
    }
    let projects = claude_code::read(dir.path(), None, true, &[]);
    let redactor = Redactor::new(Labels::for_locale("en"), Vec::new());
    let digest = digest::build(
        projects,
        &own::Extras::default(),
        &redactor,
        words("en"),
        &TimeZone::UTC,
    );
    assert!(
        digest.text.chars().count() <= IMPORT_MAX_CHARS,
        "{}",
        digest.text.chars().count()
    );
    assert_eq!(
        digest.scope,
        [
            "C:\\work\\p5",
            "C:\\work\\p4",
            "C:\\work\\p3",
            "C:\\work\\p2"
        ]
    );
    assert_eq!(digest.cut, ["C:\\work\\p1", "C:\\work\\p0"]);
    assert!(digest.text.contains("Left out for size"));
}
