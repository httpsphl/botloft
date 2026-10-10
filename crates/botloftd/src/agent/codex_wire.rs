//! The conversation with `codex app-server` (spec 30): JSON-RPC lines on
//! stdin and stdout. The daemon writes Botloft's own neutral turn line to the
//! process (one JSON object with `"event": "user"`); [`CodexControl`] turns it
//! into a `turn/start` once the thread exists and no turn runs, and the
//! [`super::codex::CodexDecoder`] reads the answers through the same
//! [`Shared`] state.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use serde_json::{Value, json};

use super::AttachInput;
use crate::runtime::ProcessControl;

/// What a request that was sent is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pending {
    Initialize,
    Account,
    ThreadStart,
    ThreadResume,
    Turn,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Phase {
    /// `initialize` or the thread request is still on its way.
    Starting,
    Ready {
        thread: String,
    },
    /// The thread could not be made: nothing will run.
    Failed,
}

/// One thing for the thread to do.
enum Work {
    /// A message: the input of a `turn/start`.
    Turn(Vec<Value>),
    /// Compact the conversation (it runs as a turn of its own).
    Compact,
}

pub(super) struct State {
    next_id: u64,
    phase: Phase,
    pending: HashMap<u64, Pending>,
    /// What waits for the thread or for the running turn.
    queue: VecDeque<Work>,
    turn_active: bool,
}

pub(super) struct Shared {
    real: Box<dyn ProcessControl>,
    input: AttachInput,
    state: Mutex<State>,
}

impl Shared {
    pub(super) fn begin(real: Box<dyn ProcessControl>, input: AttachInput) -> Arc<Self> {
        let shared = Arc::new(Self {
            real,
            input,
            state: Mutex::new(State {
                next_id: 1,
                phase: Phase::Starting,
                pending: HashMap::new(),
                queue: VecDeque::new(),
                turn_active: false,
            }),
        });
        {
            let mut state = shared.lock();
            shared.request(
                &mut state,
                "initialize",
                json!({ "clientInfo": { "name": "botloft", "version": env!("CARGO_PKG_VERSION") } }),
                Pending::Initialize,
            );
        }
        shared
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn send(&self, message: &Value) {
        let mut line = message.to_string().into_bytes();
        line.push(b'\n');
        // A process that is gone has its exit event on the way.
        let _ = self.real.write(Bytes::from(line));
    }

    fn request(&self, state: &mut State, method: &str, params: Value, kind: Pending) {
        let id = state.next_id;
        state.next_id += 1;
        state.pending.insert(id, kind);
        self.send(&json!({ "id": id, "method": method, "params": params }));
    }

    /// What the thread is made with: where it works, that it asks for
    /// nothing yet (A3a), the model, and Botloft's MCP server with the
    /// token of this process.
    fn thread_params(&self) -> Value {
        let mut params = json!({
            "cwd": self.input.workspace,
            // A plan only reads; otherwise whatever needs more than reading
            // is asked, and Botloft asks the owner (spec 30).
            "approvalPolicy": if self.input.may_ask { "on-request" } else { "never" },
            "sandbox": "read-only",
            "config": {
                "mcp_servers": {
                    "botloft": {
                        "url": format!("http://127.0.0.1:{}/mcp", self.input.port),
                        "http_headers": { "Authorization": format!("Bearer {}", self.input.token) },
                    },
                },
            },
        });
        if let Some(model) = &self.input.model {
            params["model"] = json!(model);
        }
        for (slug, entry) in &self.input.connected {
            params["config"]["mcp_servers"][slug] = entry.clone();
        }
        params
    }

    /// A message from the daemon (the neutral line): queued, and sent when
    /// the thread is ready and nothing runs.
    pub(super) fn enqueue(&self, line: &[u8]) -> io::Result<()> {
        let event: Value = serde_json::from_slice(line)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        if event["event"] == "compact" {
            let mut state = self.lock();
            state.queue.push_back(Work::Compact);
            self.pump(&mut state);
            return Ok(());
        }
        // Text as it is; an image as the data URL `turn/start` takes.
        let input: Vec<Value> = event["message"]["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|block| match block["type"].as_str() {
                        Some("text") => Some(json!({ "type": "text", "text": block["text"] })),
                        Some("image") => Some(json!({
                            "type": "image",
                            "url": format!(
                                "data:{};base64,{}",
                                block["media_type"].as_str().unwrap_or("image/png"),
                                block["data"].as_str().unwrap_or_default()
                            ),
                        })),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut state = self.lock();
        state.queue.push_back(Work::Turn(input));
        self.pump(&mut state);
        Ok(())
    }

    fn pump(&self, state: &mut State) {
        let Phase::Ready { thread } = &state.phase else {
            return;
        };
        if state.turn_active {
            return;
        }
        let Some(work) = state.queue.pop_front() else {
            return;
        };
        let input = match work {
            Work::Turn(input) => input,
            Work::Compact => {
                let params = json!({ "threadId": thread });
                state.turn_active = true;
                self.request(state, "thread/compact/start", params, Pending::Compact);
                return;
            }
        };
        let mut params = json!({
            "threadId": thread,
            "input": input,
        });
        if let Some(effort) = &self.input.effort {
            params["effort"] = json!(effort);
        }
        state.turn_active = true;
        self.request(state, "turn/start", params, Pending::Turn);
    }

    /// A response line (`id` and `result` or `error`). Returns the id of a
    /// thread that became ready, for the caller to remember.
    pub(super) fn answered(&self, event: &Value) -> Answer {
        let Some(id) = event["id"].as_u64() else {
            return Answer::None;
        };
        let mut state = self.lock();
        let Some(kind) = state.pending.remove(&id) else {
            return Answer::None;
        };
        let failed = !event["error"].is_null();
        match kind {
            Pending::Initialize if !failed => {
                self.send(&json!({ "method": "initialized", "params": {} }));
                // Whether Codex is signed in: asked in parallel with the thread.
                self.request(&mut state, "account/read", json!({}), Pending::Account);
                let params = self.thread_params();
                match &self.input.resume {
                    Some(thread) => {
                        let mut params = params;
                        params["threadId"] = json!(thread);
                        self.request(&mut state, "thread/resume", params, Pending::ThreadResume);
                    }
                    None => self.request(&mut state, "thread/start", params, Pending::ThreadStart),
                }
                Answer::None
            }
            Pending::ThreadResume if failed => {
                // The conversation is gone: a new one takes its place.
                let params = self.thread_params();
                self.request(&mut state, "thread/start", params, Pending::ThreadStart);
                Answer::Lost
            }
            Pending::ThreadStart | Pending::ThreadResume if !failed => {
                let thread = event["result"]["thread"]["id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                state.phase = Phase::Ready {
                    thread: thread.clone(),
                };
                self.pump(&mut state);
                Answer::Thread {
                    thread,
                    model: event["result"]["model"].as_str().map(str::to_owned),
                }
            }
            Pending::Account => {
                // `account` is null when nobody is signed in.
                if !failed && event["result"]["account"].is_null() {
                    Answer::SignedOut
                } else {
                    Answer::None
                }
            }
            Pending::Compact if failed => {
                state.turn_active = false;
                self.pump(&mut state);
                Answer::None
            }
            Pending::Compact => Answer::None,
            Pending::Turn if failed => {
                state.turn_active = false;
                self.pump(&mut state);
                Answer::TurnRefused(event["error"]["message"].as_str().map(str::to_owned))
            }
            Pending::Initialize | Pending::ThreadStart | Pending::ThreadResume => {
                state.phase = Phase::Failed;
                Answer::Failed(event["error"]["message"].as_str().map(str::to_owned))
            }
            Pending::Turn => Answer::None,
        }
    }

    /// The turn ended: the next queued message may go.
    pub(super) fn turn_done(&self) {
        let mut state = self.lock();
        state.turn_active = false;
        self.pump(&mut state);
    }

    /// Refuses a request of the server that Botloft does not take.
    pub(super) fn refuse(&self, id: &Value, message: &str) {
        self.send(&json!({ "id": id, "error": { "code": -32601, "message": message } }));
    }

    /// Answers a request of the server.
    pub(super) fn respond(&self, id: &Value, result: Value) {
        self.send(&json!({ "id": id, "result": result }));
    }
}

/// What an answer of the server changed for the daemon.
pub(super) enum Answer {
    None,
    /// The thread is ready; this is its id, and the model it runs on.
    Thread {
        thread: String,
        model: Option<String>,
    },
    /// The conversation to resume was not there.
    Lost,
    /// `turn/start` was refused.
    TurnRefused(Option<String>),
    /// The process cannot start a thread.
    Failed(Option<String>),
    /// Nobody is signed in to Codex.
    SignedOut,
}

/// What the supervisor writes to: Botloft's turn lines in, JSON-RPC out.
pub(super) struct CodexControl {
    pub(super) shared: Arc<Shared>,
}

impl ProcessControl for CodexControl {
    fn write(&self, data: Bytes) -> io::Result<()> {
        self.shared.enqueue(&data)
    }

    fn kill(&self) -> io::Result<()> {
        self.shared.real.kill()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StdMutex;

    use super::*;

    /// Collects what is written, as lines.
    #[derive(Default)]
    struct Sink(Arc<StdMutex<Vec<Value>>>);

    impl ProcessControl for Sink {
        fn write(&self, data: Bytes) -> io::Result<()> {
            let line = String::from_utf8_lossy(&data).into_owned();
            self.0
                .lock()
                .expect("lock")
                .push(serde_json::from_str(line.trim()).expect("json"));
            Ok(())
        }

        fn kill(&self) -> io::Result<()> {
            Ok(())
        }
    }

    fn input(resume: Option<&str>) -> AttachInput {
        AttachInput {
            workspace: "w".into(),
            port: 45710,
            token: "tok".into(),
            resume: resume.map(str::to_owned),
            model: Some("gpt-5.5".into()),
            effort: None,
            fenced: Vec::new(),
            may_ask: true,
            connected: Vec::new(),
        }
    }

    fn user(text: &str) -> Vec<u8> {
        json!({ "event": "user", "message": { "role": "user", "content": [{ "type": "text", "text": text }] } })
            .to_string()
            .into_bytes()
    }

    #[test]
    fn a_message_waits_for_the_thread_and_then_for_the_turn_before_it() {
        let sink = Sink::default();
        let written = Arc::clone(&sink.0);
        let shared = Shared::begin(Box::new(sink), input(None));
        let methods = || -> Vec<String> {
            written
                .lock()
                .expect("lock")
                .iter()
                .map(|line| line["method"].as_str().unwrap_or("response").to_owned())
                .collect()
        };
        assert_eq!(methods(), ["initialize"]);

        // Messages before the thread are held.
        shared.enqueue(&user("one")).expect("queued");
        shared.enqueue(&user("two")).expect("queued");
        assert_eq!(methods(), ["initialize"]);

        shared.answered(&json!({ "id": 1, "result": {} }));
        assert_eq!(
            methods(),
            ["initialize", "initialized", "account/read", "thread/start"]
        );
        let start = written.lock().expect("lock")[3].clone();
        assert_eq!(start["params"]["approvalPolicy"], "on-request");
        assert_eq!(start["params"]["sandbox"], "read-only");
        assert_eq!(start["params"]["model"], "gpt-5.5");
        assert_eq!(
            start["params"]["config"]["mcp_servers"]["botloft"]["http_headers"]["Authorization"],
            "Bearer tok"
        );

        shared.answered(&json!({ "id": 2, "result": { "account": { "type": "chatgpt" } } }));
        let answer = shared.answered(&json!({ "id": 3, "result": { "thread": { "id": "th-1" } } }));
        assert!(matches!(answer, Answer::Thread { thread, .. } if thread == "th-1"));
        // Only the first message goes; the second waits for its turn.
        let turns: Vec<Value> = written
            .lock()
            .expect("lock")
            .iter()
            .filter(|line| line["method"] == "turn/start")
            .cloned()
            .collect();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0]["params"]["threadId"], "th-1");
        assert_eq!(turns[0]["params"]["input"][0]["text"], "one");

        shared.turn_done();
        let texts: Vec<String> = written
            .lock()
            .expect("lock")
            .iter()
            .filter(|line| line["method"] == "turn/start")
            .map(|line| {
                line["params"]["input"][0]["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        assert_eq!(texts, ["one", "two"]);
    }

    #[test]
    fn a_conversation_that_is_gone_starts_a_new_one() {
        let sink = Sink::default();
        let written = Arc::clone(&sink.0);
        let shared = Shared::begin(Box::new(sink), input(Some("old")));
        shared.answered(&json!({ "id": 1, "result": {} }));
        let resume = written.lock().expect("lock")[3].clone();
        assert_eq!(resume["method"], "thread/resume");
        assert_eq!(resume["params"]["threadId"], "old");

        let answer = shared.answered(&json!({ "id": 3, "error": { "message": "no rollout" } }));
        assert!(matches!(answer, Answer::Lost));
        let last = written.lock().expect("lock").last().cloned().expect("line");
        assert_eq!(last["method"], "thread/start");
    }

    #[test]
    fn an_image_goes_as_a_data_url_with_the_text() {
        let sink = Sink::default();
        let written = Arc::clone(&sink.0);
        let shared = Shared::begin(Box::new(sink), input(None));
        shared.answered(&json!({ "id": 1, "result": {} }));
        shared.answered(&json!({ "id": 3, "result": { "thread": { "id": "th" } } }));
        let line = json!({ "event": "user", "message": { "role": "user", "content": [
            { "type": "text", "text": "look" },
            { "type": "image", "media_type": "image/png", "data": "AAAA" },
        ] } })
        .to_string();
        shared.enqueue(line.as_bytes()).expect("queued");
        let turn = written
            .lock()
            .expect("lock")
            .iter()
            .find(|line| line["method"] == "turn/start")
            .cloned()
            .expect("turn");
        assert_eq!(turn["params"]["input"][0]["text"], "look");
        assert_eq!(turn["params"]["input"][1]["type"], "image");
        assert_eq!(
            turn["params"]["input"][1]["url"],
            "data:image/png;base64,AAAA"
        );
    }
}
