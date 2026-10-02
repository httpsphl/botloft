//! A connection watching one bot's browser (spec 21.7): `browser.watch`
//! and `browser.unwatch`, the frames that follow, and the size its panel
//! gives the page. The same connection reloads the page, takes the browser
//! into the owner's hands, moves between its tabs and gives it back (spec
//! 21.10).

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    ApprovalsAnswerParams, BrowserControlParams, BrowserInputParams, BrowserOpenParams,
    BrowserResizeParams, BrowserTabParams, BrowserWatchParams, error_code, method, notification,
};
use serde::Serialize;
use serde_json::Value;

use super::dispatch::parse;
use super::jsonrpc::{self, RpcError};
use crate::approvals;
use crate::browser::{Hands, InputError, Watching};
use crate::service::{ApiError, bots};
use crate::state::Daemon;

/// The most room a panel may say it has, in pixels each way.
const ROOM_MAX: u32 = 10_000;

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
            method::BROWSER_RESIZE => return self.resize(daemon, &parse(params)?),
            method::BROWSER_TAKE => return self.take(daemon, parse(params)?),
            method::BROWSER_RELEASE => return self.release(daemon, &parse(params)?),
            method::BROWSER_INPUT => return self.input(parse(params)?),
            method::BROWSER_RELOAD => return self.reload(daemon, &parse(params)?),
            method::BROWSER_NEW_TAB => return self.new_tab(&parse(params)?),
            method::BROWSER_SWITCH_TAB => return self.switch_tab(&parse(params)?),
            method::BROWSER_OPEN => return self.open(&parse(params)?),
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

    fn watches(&self, bot: &BotId) -> bool {
        self.current
            .as_ref()
            .is_some_and(|watching| watching.bot == *bot)
    }

    /// The page takes the shape of the room this connection's panel has.
    fn resize(&self, daemon: &Daemon, params: &BrowserResizeParams) -> Result<Value, RpcError> {
        if !self.watches(&params.bot_id) {
            return Err(
                ApiError::Conflict("watch this browser before sizing it".to_owned()).into(),
            );
        }
        let fits = |side: u32| (1..=ROOM_MAX).contains(&side);
        if !fits(params.width) || !fits(params.height) {
            return Err(
                ApiError::validation(format!("width and height go from 1 to {ROOM_MAX}")).into(),
            );
        }
        daemon
            .browsers
            .resize(&params.bot_id, params.width, params.height);
        Ok(Value::Null)
    }

    fn take(&mut self, daemon: &Daemon, params: BrowserControlParams) -> Result<Value, RpcError> {
        let bot = &params.bot_id;
        if !self.watches(bot) {
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
                    always: None,
                };
                // It may have been answered in the chat a moment before.
                let _ = approvals::answer(daemon, done);
            }
        }
        to_value(&daemon.browsers.view(bot).state)
    }

    /// The owner's hold on the bot's browser, if this connection has it.
    fn hands(&self, bot: &BotId) -> Result<&Hands, RpcError> {
        self.hands
            .as_ref()
            .filter(|hands| hands.bot == *bot)
            .ok_or_else(|| ApiError::Conflict("take the browser before using it".to_owned()).into())
    }

    fn input(&self, params: BrowserInputParams) -> Result<Value, RpcError> {
        made(self.hands(&params.bot_id)?.send(params.input))
    }

    /// Reloads the page of the watched browser; it needs no hold.
    fn reload(&self, daemon: &Daemon, params: &BrowserControlParams) -> Result<Value, RpcError> {
        let bot = &params.bot_id;
        if !self.watches(bot) {
            return Err(
                ApiError::Conflict("watch this browser before reloading it".to_owned()).into(),
            );
        }
        if !daemon.browsers.reload(bot) {
            return Err(ApiError::Conflict("the browser is not open".to_owned()).into());
        }
        Ok(Value::Null)
    }

    fn new_tab(&self, params: &BrowserControlParams) -> Result<Value, RpcError> {
        made(self.hands(&params.bot_id)?.new_tab())
    }

    fn switch_tab(&self, params: &BrowserTabParams) -> Result<Value, RpcError> {
        made(self.hands(&params.bot_id)?.switch_tab(&params.tab_id))
    }

    fn open(&self, params: &BrowserOpenParams) -> Result<Value, RpcError> {
        made(self.hands(&params.bot_id)?.open(&params.url))
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
        Some(jsonrpc::notification(notification::BROWSER_FRAME, &*frame))
    }
}

/// What the owner did is on its way to the browser, or why it is not.
fn made(queued: Result<(), InputError>) -> Result<Value, RpcError> {
    queued.map_err(|err| match err {
        InputError::Invalid | InputError::Address => ApiError::validation(err.to_string()),
        InputError::NoTab => ApiError::NotFound(err.to_string()),
        InputError::NotHeld | InputError::Closed => ApiError::Conflict(err.to_string()),
    })?;
    Ok(Value::Null)
}

fn to_value(value: &impl Serialize) -> Result<Value, RpcError> {
    serde_json::to_value(value)
        .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
}
