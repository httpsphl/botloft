//! An agent's keys (spec 32): the values live in `secrets\keys\<bot_id>.json`,
//! readable only by the current user, and reach the agent only as variables
//! of its process. Every text the daemon keeps about the agent has those
//! values replaced by `[chave NAME]` first. Values never go to the
//! database, the log or the app.

use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use botloft_core::ids::BotId;
use botloft_core::keys::redact;
use botloft_core::protocol::ChatBody;
use tracing::warn;

use crate::platform;
use crate::state::Daemon;

/// A key's name and value.
pub type Pairs = Vec<(String, String)>;

fn folder(secrets_dir: &Path) -> PathBuf {
    secrets_dir.join("keys")
}

fn file(secrets_dir: &Path, bot: &BotId) -> PathBuf {
    folder(secrets_dir).join(format!("{bot}.json"))
}

fn read(secrets_dir: &Path, bot: &BotId) -> io::Result<BTreeMap<String, String>> {
    match std::fs::read(file(secrets_dir, bot)) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(err) => Err(err),
    }
}

fn write(secrets_dir: &Path, bot: &BotId, keys: &BTreeMap<String, String>) -> io::Result<()> {
    let path = file(secrets_dir, bot);
    if keys.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    let folder = folder(secrets_dir);
    std::fs::create_dir_all(&folder)?;
    platform::restrict_to_current_user(&folder)?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec(keys).map_err(io::Error::other)?)?;
    platform::restrict_to_current_user(&tmp)?;
    std::fs::rename(&tmp, &path)
}

/// The keys of every agent, read once from their files and kept in memory,
/// since every chat item looks them up.
#[derive(Default)]
pub struct KeyVault {
    cache: Mutex<HashMap<BotId, Arc<Pairs>>>,
}

impl KeyVault {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Arc<Pairs>>> {
        self.cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The agent's keys; none when its file cannot be read.
    pub fn pairs(&self, secrets_dir: &Path, bot: &BotId) -> Arc<Pairs> {
        if let Some(pairs) = self.lock().get(bot) {
            return Arc::clone(pairs);
        }
        let pairs = match read(secrets_dir, bot) {
            Ok(keys) => Arc::new(keys.into_iter().collect::<Pairs>()),
            Err(err) => {
                warn!(bot = %bot, "could not read the agent's keys: {err}");
                Arc::new(Pairs::new())
            }
        };
        self.lock().insert(bot.clone(), Arc::clone(&pairs));
        pairs
    }

    /// Saves `value` under `name` for the agent.
    pub fn set(&self, secrets_dir: &Path, bot: &BotId, name: &str, value: &str) -> io::Result<()> {
        let mut keys = read(secrets_dir, bot)?;
        keys.insert(name.to_owned(), value.to_owned());
        write(secrets_dir, bot, &keys)?;
        self.lock().remove(bot);
        Ok(())
    }

    /// Forgets one key of the agent, or all of them with `None`.
    pub fn remove(&self, secrets_dir: &Path, bot: &BotId, name: Option<&str>) -> io::Result<()> {
        let mut keys = match name {
            Some(_) => read(secrets_dir, bot)?,
            None => BTreeMap::new(),
        };
        if let Some(name) = name {
            keys.remove(name);
        }
        write(secrets_dir, bot, &keys)?;
        self.lock().remove(bot);
        Ok(())
    }
}

/// The variables the agent starts with (spec 32.4).
pub fn environment(daemon: &Daemon, bot: &BotId) -> Vec<(OsString, OsString)> {
    daemon
        .keys
        .pairs(&daemon.paths.secrets(), bot)
        .iter()
        .map(|(name, value)| (OsString::from(name), OsString::from(value)))
        .collect()
}

/// Replaces the agent's key values in every text of `body` (spec 32.6).
pub fn redact_body(daemon: &Daemon, bot: &BotId, body: &mut ChatBody) {
    let pairs = daemon.keys.pairs(&daemon.paths.secrets(), bot);
    if !pairs.is_empty() {
        redact_with(&pairs, body);
    }
}

/// `text` with the agent's key values replaced.
pub fn redact_text(daemon: &Daemon, bot: &BotId, text: &mut String) {
    let pairs = daemon.keys.pairs(&daemon.paths.secrets(), bot);
    if let Some(clean) = redact(text, &pairs) {
        *text = clean;
    }
}

fn redact_with(pairs: &Pairs, body: &mut ChatBody) {
    let clean = |text: &mut String| {
        if let Some(out) = redact(text, pairs) {
            *text = out;
        }
    };
    let clean_opt = |text: &mut Option<String>| {
        if let Some(text) = text
            && let Some(out) = redact(text, pairs)
        {
            *text = out;
        }
    };
    match body {
        ChatBody::Reply(reply) => clean(&mut reply.text),
        ChatBody::Tool(tool) => {
            clean(&mut tool.summary);
            clean(&mut tool.input);
            clean_opt(&mut tool.explanation);
            clean_opt(&mut tool.output);
        }
        ChatBody::Approval(approval) => {
            clean(&mut approval.summary);
            clean(&mut approval.input);
            clean_opt(&mut approval.explanation);
        }
        ChatBody::Inbound(inbound) => clean(&mut inbound.message.body),
        ChatBody::Notice(notice) => clean(&mut notice.text),
        ChatBody::KeyRequest(request) => clean(&mut request.why),
        ChatBody::Question(_) | ChatBody::Turn(_) => {}
    }
}

/// Writes  with the agent's keys, or removes it
/// without any (spec 32.4). Only the names and descriptions: never a value.
pub fn write_rules(workspace: &Path, keys: &[botloft_core::protocol::AgentKey]) -> io::Result<()> {
    let path = workspace.join(".claude").join("rules").join("keys.md");
    let present: Vec<_> = keys.iter().filter(|key| !key.missing).collect();
    if present.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        };
    }
    let mut text = String::from(
        "# Your keys\n\nThe owner saved these keys for you. Each one is in your environment \
         under its name: use it by the name (`$env:NAME` in PowerShell, `$NAME` in Bash), \
         never by its value.\n\n",
    );
    for key in present {
        text.push_str(&format!("- `{}`", key.name));
        if !key.description.is_empty() {
            text.push_str(&format!(": {}", key.description));
        }
        text.push('\n');
    }
    text.push_str(
        "\nNever write a key's value in a reply, a file, a message to another bot, or a \
         command that prints it (echo, Write-Output). If a key is missing or does not work, \
         ask the owner for it with ask_key, by its name.\n",
    );
    std::fs::create_dir_all(path.parent().unwrap_or(workspace))?;
    std::fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{ReplyItem, ToolItem, ToolStatus};

    use super::*;

    #[test]
    fn keys_are_saved_per_agent_and_forgotten() {
        let dir = tempfile::tempdir().expect("dir");
        let vault = KeyVault::default();
        let (a, b) = (BotId::generate(), BotId::generate());
        vault
            .set(dir.path(), &a, "WA_TOKEN", "wa-1234567890")
            .expect("set");
        vault
            .set(dir.path(), &a, "B_KEY", "b-1234567890")
            .expect("set");
        assert_eq!(vault.pairs(dir.path(), &a).len(), 2);
        assert!(vault.pairs(dir.path(), &b).is_empty());
        vault.remove(dir.path(), &a, Some("B_KEY")).expect("remove");
        assert_eq!(
            *vault.pairs(dir.path(), &a),
            vec![("WA_TOKEN".to_owned(), "wa-1234567890".to_owned())]
        );
        vault.remove(dir.path(), &a, None).expect("all");
        assert!(vault.pairs(dir.path(), &a).is_empty());
        assert!(!file(dir.path(), &a).exists(), "no file is left");
    }

    #[test]
    fn every_text_of_an_item_loses_the_values() {
        let pairs = vec![("WA_TOKEN".to_owned(), "wa-1234567890".to_owned())];
        let mut tool = ChatBody::Tool(ToolItem {
            tool_use_id: "toolu_1".into(),
            name: "Bash".into(),
            summary: "echo wa-1234567890".into(),
            explanation: Some("prints wa-1234567890".into()),
            input: r#"{"command":"echo wa-1234567890"}"#.into(),
            status: ToolStatus::Done,
            output: Some("wa-1234567890".into()),
            file: None,
            lines: None,
        });
        redact_with(&pairs, &mut tool);
        let text = serde_json::to_string(&tool).expect("json");
        assert!(!text.contains("wa-1234567890"), "{text}");
        assert!(text.contains("[chave WA_TOKEN]"), "{text}");
        let mut reply = ChatBody::Reply(ReplyItem {
            text: "the token is wa-1234567890".into(),
        });
        redact_with(&pairs, &mut reply);
        assert_eq!(
            reply,
            ChatBody::Reply(ReplyItem {
                text: "the token is [chave WA_TOKEN]".into()
            })
        );
    }
}
