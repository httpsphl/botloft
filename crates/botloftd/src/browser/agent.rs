//! What the browser tells sites it is (spec 21.3): the user agent of the
//! Edge it runs without the word `Headless`, with the brands and platform a
//! site reads from `navigator.userAgentData`. Overriding the user agent
//! alone would leave those empty, which no common browser does.

use std::time::Duration;

use serde_json::{Value, json};
use tokio::sync::mpsc;
use tracing::debug;

use super::BrowserError;
use super::cdp::{Cdp, CdpEvent};

/// A page of the browser's own that has `navigator.userAgentData`, which
/// only secure pages have (`about:blank` does not).
const SECURE_PAGE: &str = "chrome://version";
/// What `navigator.userAgentData` says, once the page has it.
const READ_METADATA: &str = "navigator.userAgentData ? navigator.userAgentData\
    .getHighEntropyValues(['architecture', 'bitness', 'fullVersionList', 'model', \
    'platformVersion', 'wow64']) : null";
/// Tries while the page loads, 50 ms apart.
const TRIES: usize = 40;

/// The params of `Emulation.setUserAgentOverride` for every tab. Run before
/// anything else reads `events`: the page it reads from opens and closes
/// here, and its events are dropped.
pub async fn read(
    cdp: &Cdp,
    events: &mut mpsc::UnboundedReceiver<CdpEvent>,
) -> Result<Value, BrowserError> {
    let version = cdp.call(None, "Browser.getVersion", json!({})).await?;
    let user_agent = version["userAgent"]
        .as_str()
        .unwrap_or_default()
        .replace("HeadlessChrome", "Chrome");
    let mut params = json!({ "userAgent": user_agent });
    match metadata(cdp).await {
        Ok(Some(metadata)) => params["userAgentMetadata"] = metadata,
        Ok(None) => debug!("browser: the user agent data did not come"),
        Err(err) => debug!("browser: reading the user agent data: {err}"),
    }
    while events.try_recv().is_ok() {}
    Ok(params)
}

/// The brands and platform a secure page reads, in the shape
/// `Emulation.UserAgentMetadata` takes.
async fn metadata(cdp: &Cdp) -> Result<Option<Value>, BrowserError> {
    let created = cdp
        .call(
            None,
            "Target.createTarget",
            json!({ "url": SECURE_PAGE, "background": true }),
        )
        .await?;
    let target = created["targetId"].clone();
    let attached = cdp
        .call(
            None,
            "Target.attachToTarget",
            json!({ "targetId": target, "flatten": true }),
        )
        .await;
    let read = match attached {
        Ok(attached) => {
            let session = attached["sessionId"].as_str().unwrap_or_default();
            let read = evaluate(cdp, session).await;
            let params = json!({ "sessionId": session });
            let _ = cdp.call(None, "Target.detachFromTarget", params).await;
            read
        }
        Err(err) => Err(err.into()),
    };
    let _ = cdp
        .call(None, "Target.closeTarget", json!({ "targetId": target }))
        .await;
    // It closes a moment after the answer: still there, the tabs the daemon
    // takes next would count it.
    for _ in 0..TRIES {
        let targets = cdp.call(None, "Target.getTargets", json!({})).await?;
        let open = targets["targetInfos"]
            .as_array()
            .is_some_and(|all| all.iter().any(|info| info["targetId"] == target));
        if !open {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    read
}

async fn evaluate(cdp: &Cdp, session: &str) -> Result<Option<Value>, BrowserError> {
    let params =
        json!({ "expression": READ_METADATA, "awaitPromise": true, "returnByValue": true });
    for _ in 0..TRIES {
        // The page may still be blank, or its context gone as it loads.
        if let Ok(mut read) = cdp
            .call(Some(session), "Runtime.evaluate", params.clone())
            .await
        {
            let value = read["result"]["value"].take();
            if value.get("brands").is_some() {
                return Ok(Some(value));
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Ok(None)
}
