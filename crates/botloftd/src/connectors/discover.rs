//! Finding the owner's claude.ai connectors (spec 34.3): a Claude Code
//! process with no turn, in a folder of its own and without
//! `--strict-mcp-config`, asked `mcp_status` until the connectors settle.
//! Nothing reaches the model.

use std::ffi::OsString;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use botloft_core::protocol::{Connector, ConnectorState};
use serde_json::{Value, json};

use crate::platform::ProcessJob;

/// How often the process is asked, and for how long at most.
const EVERY: Duration = Duration::from_secs(2);
const LIMIT: Duration = Duration::from_secs(30);
/// The prefix Claude Code puts before a connector's name.
const PREFIX: &str = "claude.ai ";

/// The connectors of the account Claude Code signs in with, by name.
pub(super) fn discover(
    program: &Path,
    env: &[(OsString, OsString)],
    dir: &Path,
) -> io::Result<Vec<Connector>> {
    std::fs::create_dir_all(dir)?;
    let mut command = Command::new(program);
    command
        .args([
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--setting-sources",
            "project,local",
        ])
        .current_dir(dir)
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
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("no stdout"))?;
    let (answers, heard) = mpsc::channel::<Vec<Value>>();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let Ok(event) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            if let Some(servers) = event["response"]["response"]["mcpServers"].as_array()
                && answers.send(servers.clone()).is_err()
            {
                break;
            }
        }
    });

    let started = Instant::now();
    let mut found = Vec::new();
    let mut asked = 0u32;
    while started.elapsed() < LIMIT {
        asked += 1;
        let request = json!({
            "type": "control_request",
            "request_id": format!("botloft-connectors-{asked}"),
            "request": { "subtype": "mcp_status" },
        });
        if writeln!(stdin, "{request}").is_err() {
            break;
        }
        if let Ok(servers) = heard.recv_timeout(EVERY) {
            found = connectors(&servers);
            // The list comes in a few seconds after the start; it is done
            // when it has come and nothing in it is still connecting.
            let settled = !found.is_empty()
                && found
                    .iter()
                    .all(|connector| connector.state != ConnectorState::Pending);
            if settled {
                break;
            }
            thread::sleep(EVERY);
        }
    }
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
    Ok(found)
}

/// The claude.ai entries of an `mcp_status` answer.
pub(super) fn connectors(servers: &[Value]) -> Vec<Connector> {
    let mut found: Vec<Connector> = servers
        .iter()
        .filter(|server| server["scope"] == "claudeai")
        .filter_map(|server| {
            let name = server["name"].as_str()?;
            let name = name.strip_prefix(PREFIX).unwrap_or(name).trim();
            (!name.is_empty()).then(|| Connector {
                name: name.to_owned(),
                state: match server["status"].as_str() {
                    Some("connected") => ConnectorState::Connected,
                    Some("needs-auth") => ConnectorState::NeedsAuth,
                    Some("pending") => ConnectorState::Pending,
                    _ => ConnectorState::Failed,
                },
            })
        })
        .collect();
    found.sort_by_key(|connector| connector.name.to_lowercase());
    found
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn only_the_claude_ai_entries_count_by_their_plain_name() {
        // Trimmed from Claude Code 2.1.309's answer on 2026-11-02.
        let servers = [
            json!({ "name": "botloft", "status": "connected", "scope": "dynamic" }),
            json!({ "name": "planted", "status": "pending", "scope": "project" }),
            json!({ "name": "claude.ai Google Calendar", "status": "connected", "scope": "claudeai" }),
            json!({ "name": "claude.ai Figma", "status": "needs-auth", "scope": "claudeai" }),
            json!({ "name": "claude.ai Canva", "status": "pending", "scope": "claudeai" }),
        ];
        let found = connectors(&servers);
        let seen: Vec<_> = found
            .iter()
            .map(|connector| (connector.name.as_str(), connector.state))
            .collect();
        assert_eq!(
            seen,
            [
                ("Canva", ConnectorState::Pending),
                ("Figma", ConnectorState::NeedsAuth),
                ("Google Calendar", ConnectorState::Connected),
            ]
        );
    }

    /// Run by hand: `BOTLOFT_TEST_CLAUDE=<claude.exe> cargo test -p botloftd
    /// real_claude_code -- --ignored --nocapture`. It uses the account Claude
    /// Code signs in with and prints only names and states.
    #[test]
    #[ignore = "runs the real Claude Code with the owner's account"]
    fn the_real_claude_code_lists_the_connectors() {
        let program = std::env::var_os("BOTLOFT_TEST_CLAUDE").expect("BOTLOFT_TEST_CLAUDE");
        let env: Vec<_> = std::env::vars_os()
            .filter(|(name, _)| {
                let name = name.to_string_lossy().to_uppercase();
                !name.starts_with("CLAUDE") && !name.starts_with("ANTHROPIC")
            })
            .collect();
        let dir = tempfile::tempdir().expect("tempdir");
        let found = discover(Path::new(&program), &env, dir.path()).expect("discover");
        let seen: Vec<_> = found
            .iter()
            .map(|connector| format!("{}|{}", connector.name, connector.state))
            .collect();
        eprintln!("connectors: {seen:?}");
        assert!(!found.is_empty());
    }
}
