//! Streaming terminal output to one connection (spec 8).

use std::collections::HashMap;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BotIdParams, TerminalAttachParams, TerminalAttachResult, TerminalData, notification,
};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

use super::dispatch::parse;
use super::jsonrpc::{self, Request, RpcError};
use crate::service;
use crate::state::Daemon;
use crate::terminal::Attachment;

/// The terminals this connection watches, one forwarding task each.
#[derive(Default)]
pub struct Streams {
    tasks: HashMap<BotId, JoinHandle<()>>,
}

impl Streams {
    fn replace(&mut self, bot: BotId, task: JoinHandle<()>) {
        if let Some(old) = self.tasks.insert(bot, task) {
            old.abort();
        }
    }

    pub fn stop_all(&mut self) {
        for (_, task) in self.tasks.drain() {
            task.abort();
        }
    }
}

/// Answers `terminal.attach` and then streams the replay and live output.
/// Returns a frame only when there is no stream to start.
pub async fn attach(
    daemon: &Daemon,
    request: Request,
    outbox: &mpsc::Sender<String>,
    streams: &mut Streams,
) -> Option<String> {
    let id = request.id.unwrap_or(Value::Null);
    let attached = parse::<TerminalAttachParams>(request.params).and_then(|params| {
        let bot = params.bot_id.clone();
        service::terminal::attach(daemon, params)
            .map(|attachment| (bot, attachment))
            .map_err(RpcError::from)
    });
    let (bot, attachment) = match attached {
        Ok(attached) => attached,
        Err(err) => return Some(jsonrpc::failure(&id, &err)),
    };
    let point = attachment.point;
    let result = TerminalAttachResult {
        generation: point.generation,
        offset: point.offset,
        reset: point.reset,
    };
    let frame = jsonrpc::success(&id, serde_json::to_value(result).unwrap_or(Value::Null));
    if outbox.send(frame).await.is_err() {
        return None;
    }
    let task = tokio::spawn(forward(bot.clone(), attachment, outbox.clone()));
    streams.replace(bot, task);
    None
}

pub fn detach(params: Option<Value>, streams: &mut Streams) -> Result<Value, RpcError> {
    let params: BotIdParams = parse(params)?;
    if let Some(task) = streams.tasks.remove(&params.bot_id) {
        task.abort();
    }
    Ok(Value::Null)
}

/// Sends the replay, then every new chunk. A client that falls behind sees
/// the offset jump and attaches again from what it has.
async fn forward(bot: BotId, attachment: Attachment, outbox: mpsc::Sender<String>) {
    let Attachment {
        point,
        replay,
        mut live,
    } = attachment;
    if !replay.is_empty()
        && outbox
            .send(data_frame(&bot, point.generation, point.offset, &replay))
            .await
            .is_err()
    {
        return;
    }
    loop {
        let chunk = match live.recv().await {
            Ok(chunk) => chunk,
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => return,
        };
        let frame = data_frame(&bot, chunk.generation, chunk.offset, &chunk.data);
        if outbox.send(frame).await.is_err() {
            return;
        }
    }
}

fn data_frame(bot: &BotId, generation: u64, offset: u64, data: &[u8]) -> String {
    let params = TerminalData {
        bot_id: bot.clone(),
        generation,
        offset,
        data: service::terminal::encode(data),
    };
    jsonrpc::notification(
        notification::TERMINAL_DATA,
        serde_json::to_value(params).unwrap_or(Value::Null),
    )
}
