//! Reading the active tab (spec 21.6): the page reader runs in an isolated
//! world of the page, so the page's scripts neither see it nor change the
//! built-ins it uses.

use std::sync::atomic::Ordering;
use std::time::Duration;

use serde_json::Value;
use serde_json::json;

use super::BrowserError;
use super::cdp::CdpError;
use super::session::Session;

/// What reads the page (`reader.js`), and what acts on it (`actions.js`).
const READER: &str = include_str!("reader.js");
const ACTIONS: &str = include_str!("actions.js");
pub(super) const WORLD: &str = "botloft";
/// Longest page text in one answer, in characters.
pub const READ_MAX: usize = 20_000;

/// The page as the bot reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub title: String,
    pub url: String,
    pub text: String,
    /// Characters of the whole page text.
    pub total: usize,
}

impl Session {
    /// Runs `call` on the reader in the active tab's isolated world. The
    /// reader is installed on first use in each document.
    pub(super) async fn reader(&self, call: &str) -> Result<Value, BrowserError> {
        let start = self.next_ref.load(Ordering::SeqCst);
        let expression = format!("({READER})({start}, {ACTIONS}).{call}");
        let mut attempt = 0;
        loop {
            attempt += 1;
            let (session, target) = self.page()?;
            let world = self
                .cdp
                .call(
                    Some(&session),
                    "Page.createIsolatedWorld",
                    json!({ "frameId": target, "worldName": WORLD }),
                )
                .await;
            let evaluated = match world {
                Ok(world) => {
                    let params = json!({
                        "expression": expression,
                        "contextId": world["executionContextId"],
                        "returnByValue": true,
                    });
                    self.cdp
                        .call(Some(&session), "Runtime.evaluate", params)
                        .await
                }
                Err(err) => Err(err),
            };
            match evaluated {
                Ok(result) => {
                    if let Some(details) = result.get("exceptionDetails") {
                        let text = details["exception"]["description"]
                            .as_str()
                            .or_else(|| details["text"].as_str())
                            .unwrap_or("the page reader failed");
                        return Err(BrowserError::Page(
                            text.lines().next().unwrap_or_default().to_owned(),
                        ));
                    }
                    let value = result["result"]["value"].clone();
                    if let Some(next) = value["next"].as_u64() {
                        self.next_ref.fetch_max(next, Ordering::SeqCst);
                    }
                    return Ok(value);
                }
                // The page was between documents; it settles in a moment.
                Err(CdpError::Protocol { .. }) if attempt < 3 => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                Err(err) => return Err(err.into()),
            }
        }
    }

    /// The page text from character `from`, at most [`READ_MAX`].
    pub async fn read(&self, from: usize) -> Result<Reading, BrowserError> {
        let value = self.reader(&format!("read({from}, {READ_MAX})")).await?;
        let text = |key: &str| value[key].as_str().unwrap_or_default().to_owned();
        Ok(Reading {
            title: text("title"),
            url: text("url"),
            text: text("text"),
            total: value["total"].as_u64().unwrap_or_default() as usize,
        })
    }
}
