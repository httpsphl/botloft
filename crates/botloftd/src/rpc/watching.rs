//! A connection watching one bot's browser (spec 21.7): `browser.watch`
//! and `browser.unwatch`, and the frames that follow. The same connection
//! takes the browser into the owner's hands and gives it back (spec 21.10).

use botloft_core::protocol::{
    ApprovalsAnswerParams, BrowserControlParams, BrowserInputParams, BrowserWatchParams,
    error_code, method, notification,
};
use serde::Serialize;
use serde_json::Value;

use super::dispatch::parse;
use super::jsonrpc::{self, RpcError};
use crate::approvals;
use crate::browser::{Hands, InputError, Watching};
use crate::service::{ApiError, bots};
use crate::state::Daemon;

#[derive(Default)]
pub(super) struct Watch {
    current: Option<Watching>,
    /// The watched browser, when the owner took it.
    hands: Option<Hands>,
}

impl Watch {
    pub(super) fn request(
        &mut self,
        daemon: &Daemon,
        name: &str,
        params: Option<Value>,
    ) -> Result<Value, RpcError> {
        match name {
            method::BROWSER_TAKE => return self.take(daemon, parse(params)?),
            method::BROWSER_RELEASE => return self.release(daemon, &parse(params)?),
            method::BROWSER_INPUT => return self.input(parse(params)?),
            _ => {}
        }
        // Stop the one before, even if the new one is refused; the hands
        // go first, so the bot never waits for a browser nobody watches.
        self.hands = None;
        self.current = None;
        if name == method::BROWSER_UNWATCH {
            return Ok(Value::Null);
        }
        let params: BrowserWatchParams = parse(params)?;
        bots::find(&daemon.store(), &params.bot_id).map_err(RpcError::from)?;
        self.current = Some(daemon.browsers.watch(&params.bot_id));
        to_value(&daemon.browsers.view(&params.bot_id))
    }

    fn take(&mut self, daemon: &Daemon, params: BrowserControlParams) -> Result<Value, RpcError> {
        let bot = &params.bot_id;
        if self
            .current
            .as_ref()
            .is_none_or(|watching| watching.bot != *bot)
        {
            return Err(
                ApiError::Conflict("watch this browser before taking it".to_owned()).into(),
            );
        }
        if self.hands.as_ref().is_none_or(|hands| hands.bot != *bot) {
            let hands = daemon
                .browsers
                .take(bot)
                .map_err(|err| ApiError::Conflict(err.to_string()))?;
            self.hands = Some(hands);
        }
        to_value(&daemon.browsers.view(bot).state)
    }

    /// Gives the browser back; an open request for help is done.
    fn release(
        &mut self,
        daemon: &Daemon,
        params: &BrowserControlParams,
    ) -> Result<Value, RpcError> {
        let bot = &params.bot_id;
        if self.hands.as_ref().is_some_and(|hands| hands.bot == *bot) {
            self.hands = None;
            if let Some(approval_id) = daemon.browsers.take_ask(bot) {
                let done = ApprovalsAnswerParams {
                    approval_id,
                    allow: true,
                    note: None,
                    input: None,
                };
                // It may have been answered in the chat a moment before.
                let _ = approvals::answer(daemon, done);
            }
        }
        to_value(&daemon.browsers.view(bot).state)
    }

    fn input(&self, params: BrowserInputParams) -> Result<Value, RpcError> {
        let hands = self
            .hands
            .as_ref()
            .filter(|hands| hands.bot == params.bot_id)
            .ok_or_else(|| ApiError::Conflict("take the browser before using it".to_owned()))?;
        hands.send(params.input).map_err(|err| match err {
            InputError::Invalid => ApiError::validation(err.to_string()),
            InputError::NotHeld | InputError::Closed => ApiError::Conflict(err.to_string()),
        })?;
        Ok(Value::Null)
    }

    /// The watched browser's next frame, as a notification. Never resolves
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
        let params = serde_json::to_value(&*frame).unwrap_or(Value::Null);
        Some(jsonrpc::notification(notification::BROWSER_FRAME, params))
    }
}

fn to_value(value: &impl Serialize) -> Result<Value, RpcError> {
    serde_json::to_value(value)
        .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
}
