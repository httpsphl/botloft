//! The title asked of Claude Code (spec 40.6): one turn of `claude -p` in a
//! folder of its own, with Haiku, no tools, no MCP and none of the owner's
//! customizations (`--safe-mode`), on the account that already gets the
//! conversation. Checked with Claude Code 2.1.309 (spec 19).

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::Value;

use super::{TitleAsk, TitleMaker};
use crate::platform::{self, ProcessJob};
use crate::state::Daemon;

/// The longest the turn may take.
const LIMIT: Duration = Duration::from_secs(60);

const SYSTEM: &str = "You name conversations. Answer with a short title only: at most six \
                      words, no quotes, no final punctuation.";

pub struct ClaudeTitles;

impl TitleMaker for ClaudeTitles {
    fn make(&self, daemon: &Daemon, ask: &TitleAsk) -> io::Result<String> {
        let program = daemon
            .supervisor
            .claude_path()
            .ok_or_else(|| io::Error::other("Claude Code was not found"))?;
        // The owner's environment, as the bots get it, not the daemon's.
        let mut env = platform::user_environment()?;
        env.retain(|(name, _)| {
            !name
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("BOTLOFT_")
        });
        let dir = daemon.paths.home.join("titles");
        std::fs::create_dir_all(&dir)?;
        let mut command = Command::new(program);
        command
            .args([
                "-p",
                "--safe-mode",
                "--model",
                "haiku",
                "--tools",
                "",
                "--strict-mcp-config",
                "--no-session-persistence",
                "--output-format",
                "json",
                "--system-prompt",
                SYSTEM,
            ])
            .current_dir(&dir)
            .env_clear()
            .envs(env.iter().map(|(name, value)| (name, value)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        ProcessJob::prepare(&mut command);
        let job = ProcessJob::new()?;
        let mut child = command.spawn()?;
        let _ = job.assign(&child);
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("no stdin"))?;
        stdin.write_all(prompt(ask).as_bytes())?;
        drop(stdin);
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("no stdout"))?;
        let (done, read) = mpsc::channel();
        thread::spawn(move || {
            let mut text = String::new();
            let _ = done.send(stdout.read_to_string(&mut text).map(|_| text));
        });
        let text = match read.recv_timeout(LIMIT) {
            Ok(text) => text?,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::other("Claude Code took too long to name it"));
            }
        };
        let _ = child.wait();
        result_of(&text)
    }
}

/// What Claude Code reads: the start of the conversation.
fn prompt(ask: &TitleAsk) -> String {
    format!(
        "Name this conversation in at most 6 words, in the language the owner wrote in. \
         Answer with the name only.\n\nThe owner wrote:\n{}\n\nThe agent answered:\n{}",
        ask.first, ask.reply
    )
}

/// The `result` of the turn's JSON.
fn result_of(text: &str) -> io::Result<String> {
    let value: Value = serde_json::from_str(text.trim()).map_err(io::Error::other)?;
    if value["is_error"].as_bool() == Some(true) {
        return Err(io::Error::other("Claude Code could not name it"));
    }
    value["result"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| io::Error::other("no result"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_is_the_result_of_the_turn() {
        let ok = r#"{"type":"result","is_error":false,"result":"Viagem a Lisboa","num_turns":1}"#;
        assert_eq!(result_of(ok).unwrap(), "Viagem a Lisboa");
        assert!(result_of(r#"{"is_error":true,"result":"limit"}"#).is_err());
        assert!(result_of("not json").is_err());
        let asked = prompt(&TitleAsk {
            first: "Plan".into(),
            reply: "Sure".into(),
        });
        assert!(asked.ends_with("The owner wrote:\nPlan\n\nThe agent answered:\nSure"));
    }
}
