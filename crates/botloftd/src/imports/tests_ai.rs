//! The summary of a claude.ai export (spec 36.12, CR4): a test ZIP in the
//! format seen, a ZIP of something else refused, the ZIP never copied, and
//! only titles, dates, counts and the owner's first three messages.

use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use jiff::tz::TimeZone;
use serde_json::json;
use zip::write::SimpleFileOptions;

use super::claude_ai::{self, ExportError};
use super::digest_ai;
use super::redact::{Labels, Redactor};
use super::words::words;

fn message(sender: &str, text: &str) -> serde_json::Value {
    json!({ "uuid": "m", "sender": sender, "text": text, "created_at": "2026-10-01T10:00:00Z" })
}

fn zip(dir: &Path, files: &[(&str, String)]) -> PathBuf {
    let path = dir.join("export.zip");
    let mut writer = zip::ZipWriter::new(File::create(&path).expect("zip"));
    for (name, text) in files {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .expect("entry");
        writer.write_all(text.as_bytes()).expect("write");
    }
    writer.finish().expect("finish");
    path
}

fn export(dir: &Path) -> PathBuf {
    let conversations = json!([
        {
            "uuid": "c1",
            "name": "Newsletter de outubro",
            "created_at": "2026-10-03T09:00:00Z",
            "chat_messages": [
                message("human", "Escreva a newsletter da semana"),
                message("assistant", "ASSISTANT-ANSWER"),
                message("human", "Mande para ana@example.com"),
                message("human", "Mais curta"),
                message("human", "FOURTH-MESSAGE"),
            ],
        },
        {
            "uuid": "c2",
            "name": "",
            "created_at": "2026-10-10T09:00:00Z",
            "chat_messages": [
                { "sender": "human", "text": "", "content": [{ "type": "text", "text": "Escreva a newsletter da semana!" }] },
            ],
        },
        { "uuid": "c3", "name": "Sem data", "chat_messages": [message("human", "NO-DATE")] },
        { "uuid": "c4", "name": "Sem mensagens", "created_at": "2026-10-11T09:00:00Z" },
    ]);
    zip(
        dir,
        &[
            (
                "data-2026-11-03/conversations.json",
                conversations.to_string(),
            ),
            (
                "data-2026-11-03/projects.json",
                json!([{ "name": "Clínica" }]).to_string(),
            ),
            (
                "data-2026-11-03/users.json",
                json!([{ "email_address": "owner@example.com" }]).to_string(),
            ),
        ],
    )
}

#[test]
fn reads_only_what_the_summary_takes_and_leaves_broken_conversations_out() {
    let dir = tempfile::tempdir().expect("dir");
    let path = export(dir.path());
    let read = claude_ai::read(&path).expect("export");
    assert_eq!(
        read.conversations.len(),
        2,
        "no date or no messages leave it"
    );
    assert_eq!(read.projects, ["Clínica"]);
    let newest = &read.conversations[0];
    // A conversation without a title takes its first message as one.
    assert_eq!(newest.title, "Escreva a newsletter da semana!");
    let oldest = &read.conversations[1];
    assert_eq!(oldest.messages, 5);
    assert_eq!(oldest.first.len(), 3);
    // The ZIP stays where it was and nothing was extracted next to it.
    assert_eq!(std::fs::read_dir(dir.path()).expect("dir").count(), 1);
}

#[test]
fn the_summary_joins_repeats_and_swaps_what_must_not_go() {
    let dir = tempfile::tempdir().expect("dir");
    let read = claude_ai::read(&export(dir.path())).expect("export");
    let redactor = Redactor::new(Labels::for_locale("pt-BR"), Vec::new());
    let digest = digest_ai::build(&read, None, None, &redactor, words("pt-BR"), &TimeZone::UTC);
    let text = &digest.text;
    for absent in [
        "ASSISTANT-ANSWER",
        "FOURTH-MESSAGE",
        "NO-DATE",
        "owner@example.com",
        "ana@example.com",
    ] {
        assert!(!text.contains(absent), "{absent} leaked into:\n{text}");
    }
    assert!(
        text.contains("## Pedidos que se repetem entre conversas"),
        "{text}"
    );
    assert!(text.contains("2 vezes, sáb ~9h"), "{text}");
    assert!(text.contains("### Newsletter de outubro"), "{text}");
    assert!(text.contains("2026-10-03 · 5 mensagens"), "{text}");
    assert!(text.contains("- Mande para [e-mail]"), "{text}");
    assert!(text.contains("- Clínica"), "{text}");
    assert_eq!(digest.redactions.get("email"), Some(&1));

    // Only the conversations of the period.
    let october_fifth = "2026-10-05T00:00:00Z"
        .parse::<jiff::Timestamp>()
        .expect("date")
        .as_millisecond();
    let recent = digest_ai::build(
        &read,
        Some(october_fifth),
        None,
        &redactor,
        words("pt-BR"),
        &TimeZone::UTC,
    );
    assert!(
        !recent.text.contains("Newsletter de outubro"),
        "{}",
        recent.text
    );
}

#[test]
fn a_zip_of_something_else_is_refused() {
    let dir = tempfile::tempdir().expect("dir");
    let other = zip(dir.path(), &[("photos/readme.txt", "hello".to_owned())]);
    assert_eq!(claude_ai::read(&other), Err(ExportError::NotAnExport));
    let broken = zip(
        dir.path(),
        &[("conversations.json", "{\"not\": \"a list\"}".to_owned())],
    );
    assert_eq!(claude_ai::read(&broken), Err(ExportError::Format));
    let text = dir.path().join("notes.txt");
    std::fs::write(&text, "not a zip").expect("file");
    assert_eq!(claude_ai::read(&text), Err(ExportError::NotAnExport));
    assert_eq!(
        claude_ai::read(&dir.path().join("missing.zip")),
        Err(ExportError::Unreadable)
    );
}
