//! A connection watching one bot's desktop panel (spec 24.9):
//! `desktop.watch` and `desktop.unwatch`, and the pictures that follow.

use botloft_core::protocol::{DesktopBotParams, error_code, method, notification};
use serde_json::Value;

use super::dispatch::parse;
use super::jsonrpc::{self, RpcError};
use crate::desktop::DesktopWatching;
use crate::service::bots;
use crate::state::Daemon;

#[derive(Default)]
pub(super) struct DesktopWatch {
    current: Option<DesktopWatching>,
}

impl DesktopWatch {
    pub(super) fn request(
        &mut self,
        daemon: &Daemon,
        name: &str,
        params: Option<Value>,
    ) -> Result<Value, RpcError> {
        // Stop the one before, even if the new one is refused.
        self.current = None;
        if name == method::DESKTOP_UNWATCH {
            return Ok(Value::Null);
        }
        let params: DesktopBotParams = parse(params)?;
        bots::find(&daemon.store(), &params.bot_id).map_err(RpcError::from)?;
        self.current = Some(daemon.desktop.watch(&params.bot_id));
        serde_json::to_value(daemon.desktop.view(&params.bot_id))
            .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
    }

    /// The watched window's next picture, as a notification. Never resolves
    /// while nothing is watched.
    pub(super) async fn next_frame(&mut self) -> Option<String> {
        let Some(watching) = self.current.as_mut() else {
            return std::future::pending().await;
        };
        if watching.frames.changed().await.is_err() {
            self.current = None;
            return None;
        }
        let frame = watching.frames.borrow_and_update().clone()?;
        Some(jsonrpc::notification(notification::DESKTOP_FRAME, &*frame))
    }
}
