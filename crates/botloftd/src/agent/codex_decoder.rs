//! What `codex app-server` prints, as the chat, the bot's state and the
//! owner's questions (spec 30). Its requests for approval are answered here:
//! a folder Botloft fences is refused at once, a rule of the bot or the bot's
//! mode may allow, and otherwise the owner is asked in the chat as for
//! Claude Code.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    ChatBody, NoticeCode, NoticeItem, NoticeLevel, PermissionMode, TokenUsage,
};
use serde_json::{Value, json};

use super::OutputDecoder;
use super::codex_wire::{Answer, Shared};
use crate::approvals::{self, Verdict};
use crate::chat::{items, sink};
use crate::state::Daemon;

/// Reads what the server prints and tells the daemon.
pub(super) struct CodexDecoder {
    shared: Arc<Shared>,
    fenced: Vec<PathBuf>,
    thread: Option<String>,
    /// The model's context window, from `thread/tokenUsage/updated`.
    window: u64,
    /// What the thread has used (input, cached, output, reasoning), and what
    /// it had used when this turn began.
    usage: (u64, u64, u64, u64),
    base: (u64, u64, u64, u64),
    /// What each running item is about, to answer a request for it: a
    /// command, or the files a change touches.
    commands: HashMap<String, String>,
    changes: HashMap<String, Vec<(String, bool)>>,
    /// When the usage limit resets, in Unix ms, from `account/rateLimits/updated`.
    limit_resets_at: Option<i64>,
    /// The running turn is a compaction: no line of its own for it.
    compacting: bool,
}

impl CodexDecoder {
    pub(super) fn new(shared: Arc<Shared>, fenced: Vec<PathBuf>) -> Self {
        Self {
            shared,
            fenced,
            thread: None,
            window: 0,
            usage: (0, 0, 0, 0),
            base: (0, 0, 0, 0),
            commands: HashMap::new(),
            changes: HashMap::new(),
            limit_resets_at: None,
            compacting: false,
        }
    }
}

impl OutputDecoder for CodexDecoder {
    fn live_text<'a>(&mut self, event: &'a Value) -> Option<&'a str> {
        if event["method"] != "item/agentMessage/delta" {
            return None;
        }
        event["params"]["delta"]
            .as_str()
            .filter(|text| !text.is_empty())
    }

    fn handle(&mut self, daemon: &Arc<Daemon>, bot: &BotId, generation: u64, event: &Value) {
        if !daemon.supervisor.is_current(bot, generation) {
            return;
        }
        let method = event["method"].as_str();
        match (method, event.get("id")) {
            (None, Some(_)) => self.response(daemon, bot, generation, event),
            (Some(method), Some(id)) => {
                self.server_request(daemon, bot, generation, method, id, &event["params"]);
            }
            (Some(method), None) => {
                self.notification(daemon, bot, generation, method, &event["params"]);
            }
            (None, None) => {}
        }
    }
}

/// Whether `text` names a folder of `fenced` (slashes and case aside).
fn touches(fenced: &[PathBuf], text: &str) -> bool {
    let flat = |value: &str| value.replace('\\', "/").to_lowercase();
    let text = flat(text);
    fenced
        .iter()
        .map(|folder| flat(&folder.to_string_lossy()))
        .any(|folder| !folder.is_empty() && text.contains(&folder))
}

impl CodexDecoder {
    fn response(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
        match self.shared.answered(event) {
            Answer::Thread { thread, model } => {
                self.thread = Some(thread);
                if let Some(model) = model {
                    crate::service::models::reported(daemon, bot, &model);
                }
            }
            Answer::TurnRefused(message) => {
                // No turn began, so the message is read and the turn failed.
                if let Some(uuid) = daemon.supervisor.oldest_message_began(bot, generation) {
                    sink::message_read(daemon, bot, &uuid);
                }
                let error = message.unwrap_or_else(|| "the turn was refused".to_owned());
                sink::turn_finished(daemon, bot, 0, None, Some(error));
                sink::finish_turn(daemon, bot, generation, true);
            }
            Answer::Failed(message) => {
                tracing::warn!(bot = %bot, "codex could not start a thread: {message:?}");
            }
            Answer::SignedOut => self.signed_out(daemon, bot, generation),
            Answer::Lost | Answer::None => {}
        }
    }

    /// A request of the server. Botloft's own tools go through; for the
    /// rest, the owner decides (spec 30.3.2).
    fn server_request(
        &self,
        daemon: &Arc<Daemon>,
        bot: &BotId,
        generation: u64,
        method: &str,
        id: &Value,
        params: &Value,
    ) {
        let item_id = params["itemId"].as_str().unwrap_or_default().to_owned();
        let (tool, input, reply): (String, Value, fn(bool) -> Value) = match method {
            "mcpServer/elicitation/request" if params["serverName"] == "botloft" => {
                self.shared
                    .respond(id, json!({ "action": "accept", "content": null }));
                return;
            }
            "mcpServer/elicitation/request" => {
                let server = params["serverName"].as_str().unwrap_or("mcp");
                let tool = tool_in(params["message"].as_str().unwrap_or_default());
                (
                    format!("mcp__{server}__{tool}"),
                    params["_meta"]["tool_params"].clone(),
                    |allow| json!({ "action": if allow { "accept" } else { "decline" }, "content": null }),
                )
            }
            "item/commandExecution/requestApproval" => {
                let command = self
                    .commands
                    .get(&item_id)
                    .cloned()
                    .or_else(|| params["command"].as_str().map(str::to_owned))
                    .unwrap_or_default();
                (
                    "Bash".to_owned(),
                    json!({ "command": command }),
                    |allow| json!({ "decision": if allow { "accept" } else { "decline" } }),
                )
            }
            "item/fileChange/requestApproval" => {
                let files = self.changes.get(&item_id).cloned().unwrap_or_default();
                let (first, added) = files.first().cloned().unwrap_or_default();
                if files.iter().any(|(path, _)| touches(&self.fenced, path)) {
                    // Another crew's folder, or Botloft's own: not for the
                    // owner to be asked about (spec 7.5).
                    self.shared.respond(id, json!({ "decision": "decline" }));
                    return;
                }
                (
                    if added { "Write" } else { "Edit" }.to_owned(),
                    json!({ "file_path": first }),
                    |allow| json!({ "decision": if allow { "accept" } else { "decline" } }),
                )
            }
            _ => {
                self.shared.refuse(id, "not supported by Botloft yet");
                return;
            }
        };
        if tool == "Bash" && touches(&self.fenced, input["command"].as_str().unwrap_or_default()) {
            self.shared.respond(id, reply(false));
            return;
        }
        let (daemon, shared, bot, id) = (
            Arc::clone(daemon),
            Arc::clone(&self.shared),
            bot.clone(),
            id.clone(),
        );
        // The owner may take a while: the output goes on in the meantime.
        tokio::spawn(async move {
            let allow = decide(&daemon, &bot, generation, &tool, &input, &item_id).await;
            shared.respond(&id, reply(allow));
        });
    }

    fn notification(
        &mut self,
        daemon: &Daemon,
        bot: &BotId,
        generation: u64,
        method: &str,
        params: &Value,
    ) {
        match method {
            "turn/started" => {
                self.base = self.usage;
                daemon.supervisor.turn_began(bot, generation);
            }
            "item/started" => self.item_started(daemon, bot, generation, params),
            "item/completed" => self.item_completed(daemon, bot, &params["item"]),
            "thread/tokenUsage/updated" => self.token_usage(daemon, bot, &params["tokenUsage"]),
            "account/rateLimits/updated" => {
                // Seconds in Codex, milliseconds here.
                self.limit_resets_at = params["rateLimits"]["primary"]["resetsAt"]
                    .as_i64()
                    .map(|seconds| seconds * 1000)
                    .or(self.limit_resets_at);
            }
            "turn/completed" => self.turn_completed(daemon, bot, generation, &params["turn"]),
            _ => {}
        }
    }

    fn item_started(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, params: &Value) {
        let item = &params["item"];
        let id = item["id"].as_str().unwrap_or_default();
        match item["type"].as_str() {
            // The message was taken up: the oldest one written.
            Some("userMessage") => {
                if let Some(thread) = params["threadId"].as_str().or(self.thread.as_deref()) {
                    sink::session_started(daemon, bot, thread);
                }
                if let Some(uuid) = daemon.supervisor.oldest_message_began(bot, generation) {
                    sink::message_read(daemon, bot, &uuid);
                }
            }
            Some("commandExecution") => {
                let command = item["commandActions"][0]["command"]
                    .as_str()
                    .or_else(|| item["command"].as_str())
                    .unwrap_or_default();
                self.commands.insert(id.to_owned(), command.to_owned());
                sink::tool_started(daemon, bot, id, "Bash", &json!({ "command": command }));
            }
            Some("fileChange") => {
                let changes = item["changes"].as_array().cloned().unwrap_or_default();
                self.changes.insert(
                    id.to_owned(),
                    changes
                        .iter()
                        .map(|change| {
                            (
                                change["path"].as_str().unwrap_or_default().to_owned(),
                                change["kind"]["type"] == "add",
                            )
                        })
                        .collect(),
                );
                let change = changes.first().cloned().unwrap_or_default();
                let name = if change["kind"]["type"] == "add" {
                    "Write"
                } else {
                    "Edit"
                };
                sink::tool_started(
                    daemon,
                    bot,
                    id,
                    name,
                    &json!({ "file_path": change["path"] }),
                );
            }
            Some("contextCompaction") => {
                self.compacting = true;
                crate::context::status(daemon, bot, &json!({ "status": "compacting" }));
            }
            Some("mcpToolCall") => {
                let name = format!(
                    "mcp__{}__{}",
                    item["server"].as_str().unwrap_or("mcp"),
                    item["tool"].as_str().unwrap_or("tool")
                );
                sink::tool_started(daemon, bot, id, &name, &item["arguments"]);
            }
            _ => {}
        }
    }

    fn item_completed(&mut self, daemon: &Daemon, bot: &BotId, item: &Value) {
        let id = item["id"].as_str().unwrap_or_default();
        let ok = item["status"].as_str() == Some("completed");
        match item["type"].as_str() {
            Some("contextCompaction") => {
                // The owner asked, or it was full and Codex did it by itself.
                let asked = daemon.contexts.get(bot).is_some_and(|c| c.compacting);
                let trigger = if asked { "manual" } else { "auto" };
                crate::context::compacted(
                    daemon,
                    bot,
                    &json!({ "compact_metadata": { "trigger": trigger } }),
                );
            }
            Some("agentMessage") => {
                sink::reply(daemon, bot, item["text"].as_str().unwrap_or_default());
            }
            Some("commandExecution") => {
                self.commands.remove(id);
                let failed = !ok || item["exitCode"].as_i64().is_some_and(|code| code != 0);
                let output = item["aggregatedOutput"].as_str().unwrap_or_default();
                sink::tool_finished(daemon, bot, id, failed, output);
            }
            Some("fileChange") => {
                self.changes.remove(id);
                sink::tool_finished(daemon, bot, id, !ok, "");
            }
            Some("mcpToolCall") => {
                let output = item["result"]["content"]
                    .as_array()
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(|part| part["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .or_else(|| item["error"]["message"].as_str().map(str::to_owned))
                    .unwrap_or_default();
                sink::tool_finished(daemon, bot, id, !ok, &output);
            }
            _ => {}
        }
    }

    fn token_usage(&mut self, daemon: &Daemon, bot: &BotId, usage: &Value) {
        let count = |value: &Value, key: &str| value[key].as_u64().unwrap_or_default();
        self.window = count(usage, "modelContextWindow");
        self.usage = (
            count(&usage["total"], "inputTokens"),
            count(&usage["total"], "cachedInputTokens"),
            count(&usage["total"], "outputTokens"),
            count(&usage["total"], "reasoningOutputTokens"),
        );
        // The last request held the conversation up to that point.
        let last = &usage["last"];
        let held = count(last, "inputTokens") + count(last, "outputTokens");
        if held > 0 && self.window > 0 {
            crate::context::report(daemon, bot, held, self.window);
        }
    }

    fn turn_completed(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, turn: &Value) {
        let failed = turn["status"].as_str() != Some("completed");
        // A compaction ends like a turn; its notice says what happened.
        if std::mem::take(&mut self.compacting) && !failed {
            self.usage_base();
            self.shared.turn_done();
            sink::finish_turn(daemon, bot, generation, false);
            return;
        }
        match turn["error"]["codexErrorInfo"].as_str() {
            Some("unauthorized") => {
                self.signed_out(daemon, bot, generation);
                self.shared.turn_done();
                return;
            }
            Some("usageLimitExceeded") => {
                self.limit_reached(daemon, bot, generation);
                self.shared.turn_done();
                return;
            }
            _ => {}
        }
        let error = failed.then(|| {
            turn["error"]["message"]
                .as_str()
                .or_else(|| turn["status"].as_str())
                .unwrap_or("error")
                .to_owned()
        });
        let used = |now: u64, before: u64| now.saturating_sub(before);
        let (input, cached) = (
            used(self.usage.0, self.base.0),
            used(self.usage.1, self.base.1),
        );
        let output = used(self.usage.2, self.base.2) + used(self.usage.3, self.base.3);
        let tokens = (self.usage != self.base).then(|| TokenUsage {
            input: input.saturating_sub(cached),
            cache_write: 0,
            reloaded: 0,
            cache_read: cached,
            output,
        });
        let duration = turn["durationMs"].as_u64().unwrap_or_default();
        sink::turn_finished(daemon, bot, duration, tokens, error);
        self.shared.turn_done();
        sink::finish_turn(daemon, bot, generation, failed);
    }
}

impl CodexDecoder {
    /// What the thread used so far is the base of the next turn.
    fn usage_base(&mut self) {
        self.base = self.usage;
    }

    fn notice(&self, daemon: &Daemon, bot: &BotId, text: &str) {
        items::add(
            daemon,
            bot,
            ChatBody::Notice(NoticeItem {
                level: NoticeLevel::Error,
                code: Some(NoticeCode::TurnFailed),
                text: text.to_owned(),
            }),
        );
    }

    /// Nobody is signed in to Codex: only the owner can fix that, so the bot
    /// stops until they restart it (spec 7.3).
    fn signed_out(&self, daemon: &Daemon, bot: &BotId, generation: u64) {
        self.notice(
            daemon,
            bot,
            "Codex is not signed in. Sign in to Codex (codex login), then restart this bot.",
        );
        daemon.supervisor.signed_out(bot, generation);
    }

    /// The account reached its usage limit: messages wait until it resets.
    fn limit_reached(&self, daemon: &Daemon, bot: &BotId, generation: u64) {
        self.notice(
            daemon,
            bot,
            "Codex reached its usage limit. Messages wait until it resets.",
        );
        let until = self
            .limit_resets_at
            .unwrap_or_else(|| daemon.clock.now_ms() + 5 * 60 * 1000);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

/// The tool a request of an MCP server is about: the name in quotes in its
/// message ("Allow the x MCP server to run tool \"y\"?").
fn tool_in(message: &str) -> &str {
    message
        .split_once("tool \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map_or("tool", |(name, _)| name)
}

/// Whether the request goes ahead: the bot's mode, a rule of the bot, or the
/// owner (spec 10.1).
async fn decide(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    tool: &str,
    input: &Value,
    item_id: &str,
) -> bool {
    let mode = daemon
        .store()
        .bot(bot)
        .ok()
        .flatten()
        .map_or(PermissionMode::Default, |record| record.permission_mode);
    let edit = matches!(tool, "Write" | "Edit");
    if mode == PermissionMode::BypassPermissions || (mode == PermissionMode::AcceptEdits && edit) {
        return true;
    }
    matches!(
        approvals::agent_request(daemon, bot, generation, tool, input, item_id).await,
        Verdict::Allow
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_is_the_name_in_quotes_of_the_message() {
        assert_eq!(
            tool_in("Allow the linkedin MCP server to run tool \"read_profile\"?"),
            "read_profile"
        );
        assert_eq!(tool_in("something else"), "tool");
    }

    #[test]
    fn a_text_that_names_a_fenced_folder_is_caught_whatever_the_slashes_and_case() {
        let fenced = vec![PathBuf::from(r"C:\Users\Ana\Botloft\Other")];
        assert!(touches(
            &fenced,
            "type C:/users/ana/botloft/other/notes.txt"
        ));
        assert!(touches(
            &fenced,
            r"Get-Content 'C:\Users\Ana\Botloft\Other\a'"
        ));
        assert!(!touches(&fenced, r"C:\Users\Ana\Botloft\Mine\a.txt"));
        assert!(!touches(&[], "anything"));
    }
}
