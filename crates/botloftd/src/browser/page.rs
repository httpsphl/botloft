//! What a bot does in its active tab (spec 21.4): open, read, click, type,
//! choose, press, scroll, go back and look. Each action waits for the page
//! to settle (spec 21.3). Acting on an element takes two steps: `aim` finds
//! it and scrolls it into view, so the owner's cursor can get there first
//! (spec 21.7), and then the action itself.

use serde_json::{Value, json};

use super::BrowserError;
use super::keys::{self, Key};
use super::session::Session;

/// Where an action happened, for the owner's view, and what the bot should
/// know about it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Done {
    pub point: Option<(f64, f64)>,
    /// The element's name.
    pub label: Option<String>,
    /// Another element was on top of the one clicked.
    pub cover: Option<String>,
    /// What `browser_select` chose.
    pub chosen: Option<String>,
}

/// An element found and in view, before anything is done to it.
#[derive(Debug, Clone)]
pub struct Aim {
    reference: String,
    point: Value,
    pub done: Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    Down,
    Up,
    Top,
    Bottom,
}

impl Session {
    pub(super) fn page(&self) -> Result<(String, String), BrowserError> {
        self.active().ok_or(BrowserError::NoPage)
    }

    pub async fn open(&self, url: &str) -> Result<(), BrowserError> {
        let (session, _) = self.page()?;
        let marks = self.marks();
        let result = self
            .cdp
            .call(Some(&session), "Page.navigate", json!({ "url": url }))
            .await?;
        if let Some(error) = result["errorText"]
            .as_str()
            .filter(|error| !error.is_empty())
        {
            return Err(BrowserError::Navigation(error.to_owned()));
        }
        self.settle(marks).await;
        Ok(())
    }

    /// Scrolls the element into view and finds where to click it.
    pub async fn aim(&self, reference: &str) -> Result<Aim, BrowserError> {
        let point = self.reader(&format!("point({})", json!(reference))).await?;
        if point["missing"] == true {
            return Err(BrowserError::Stale(reference.to_owned()));
        }
        let label = point["label"]
            .as_str()
            .filter(|label| !label.is_empty())
            .map(str::to_owned);
        if point["hidden"] == true {
            return Err(BrowserError::Page(format!(
                "{reference} is on the page but not visible; scroll, open its menu, or pick \
                 another element"
            )));
        }
        let done = Done {
            point: point["x"].as_f64().zip(point["y"].as_f64()),
            label,
            cover: point["cover"].as_str().map(str::to_owned),
            chosen: None,
        };
        Ok(Aim {
            reference: reference.to_owned(),
            point,
            done,
        })
    }

    /// `aim` for a text field: anything else fails before the cursor goes
    /// there.
    pub async fn aim_field(&self, reference: &str) -> Result<Aim, BrowserError> {
        let aim = self.aim(reference).await?;
        if aim.point["text"] != true {
            let role = aim.point["role"].as_str().unwrap_or("element");
            return Err(BrowserError::Page(format!(
                "{reference} is a {role}, not a text field; browser_type only types into text fields"
            )));
        }
        Ok(aim)
    }

    async fn mouse_click(&self, (x, y): (f64, f64)) -> Result<(), BrowserError> {
        let (session, _) = self.page()?;
        for kind in ["mouseMoved", "mousePressed", "mouseReleased"] {
            let params = json!({ "type": kind, "x": x, "y": y, "button": "left", "clickCount": 1 });
            self.cdp
                .call(Some(&session), "Input.dispatchMouseEvent", params)
                .await?;
        }
        Ok(())
    }

    pub async fn click(&self, Aim { done, .. }: Aim) -> Result<Done, BrowserError> {
        let marks = self.marks();
        self.mouse_click(done.point.unwrap_or_default()).await?;
        self.settle(marks).await;
        Ok(done)
    }

    /// Types into a field from `aim_field`.
    pub async fn type_text(
        &self,
        aim: Aim,
        text: &str,
        submit: bool,
    ) -> Result<Done, BrowserError> {
        let Aim {
            reference, done, ..
        } = aim;
        let marks = self.marks();
        self.mouse_click(done.point.unwrap_or_default()).await?;
        let ready = self
            .reader(&format!("prepareType({})", json!(reference)))
            .await?;
        if ready["missing"] == true {
            return Err(BrowserError::Stale(reference.to_owned()));
        }
        let (session, _) = self.page()?;
        if text.is_empty() {
            if ready["hadText"] == true {
                self.press_key(keys::find("Backspace").expect("a known key"))
                    .await?;
            }
        } else {
            self.cdp
                .call(Some(&session), "Input.insertText", json!({ "text": text }))
                .await?;
        }
        if submit {
            self.press_key(keys::find("Enter").expect("a known key"))
                .await?;
        }
        self.settle(marks).await;
        Ok(done)
    }

    pub async fn choose(&self, aim: Aim, option: &str) -> Result<Done, BrowserError> {
        let Aim {
            reference,
            mut done,
            ..
        } = aim;
        let marks = self.marks();
        let result = self
            .reader(&format!("select({}, {})", json!(reference), json!(option)))
            .await?;
        if result["missing"] == true {
            return Err(BrowserError::Stale(reference.to_owned()));
        }
        if result["notSelect"] == true {
            let role = result["role"].as_str().unwrap_or("element");
            return Err(BrowserError::Page(format!(
                "{reference} is a {role}, not a select; click it and then the option instead"
            )));
        }
        if result["noOption"] == true {
            let options: Vec<&str> = result["options"]
                .as_array()
                .map(|list| list.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            return Err(BrowserError::Page(format!(
                "{reference} has no option \"{option}\". Its options: {}",
                options.join(" | ")
            )));
        }
        done.chosen = result["chosen"].as_str().map(str::to_owned);
        self.settle(marks).await;
        Ok(done)
    }

    async fn press_key(&self, key: Key) -> Result<(), BrowserError> {
        let (session, _) = self.page()?;
        let down = if key.text.is_some() {
            "keyDown"
        } else {
            "rawKeyDown"
        };
        for kind in [down, "keyUp"] {
            let mut params = json!({
                "type": kind,
                "key": key.key,
                "code": key.code,
                "windowsVirtualKeyCode": key.virtual_key,
                "nativeVirtualKeyCode": key.virtual_key,
            });
            if kind == down
                && let Some(text) = key.text
            {
                params["text"] = json!(text);
                params["unmodifiedText"] = json!(text);
            }
            self.cdp
                .call(Some(&session), "Input.dispatchKeyEvent", params)
                .await?;
        }
        Ok(())
    }

    pub async fn press(&self, key: Key) -> Result<(), BrowserError> {
        let marks = self.marks();
        self.press_key(key).await?;
        self.settle(marks).await;
        Ok(())
    }

    /// Scrolls the page; up and down go a screen with the mouse wheel.
    pub async fn scroll(&self, to: Scroll) -> Result<Done, BrowserError> {
        let (session, _) = self.page()?;
        let marks = self.marks();
        let screen = self.viewport();
        let center = (
            f64::from(screen.width) / 2.0,
            f64::from(screen.height) / 2.0,
        );
        let done = match to {
            Scroll::Down | Scroll::Up => {
                let delta =
                    f64::from(screen.height) * 0.8 * if to == Scroll::Down { 1.0 } else { -1.0 };
                let params = json!({
                    "type": "mouseWheel", "x": center.0, "y": center.1, "deltaX": 0, "deltaY": delta,
                });
                self.cdp
                    .call(Some(&session), "Input.dispatchMouseEvent", params)
                    .await?;
                Done {
                    point: Some(center),
                    ..Done::default()
                }
            }
            Scroll::Top | Scroll::Bottom => {
                let end = if to == Scroll::Top { "top" } else { "bottom" };
                self.reader(&format!("scrollEnd({})", json!(end))).await?;
                Done::default()
            }
        };
        self.settle(marks).await;
        Ok(done)
    }

    /// Scrolls until the element is in the middle of the screen.
    pub async fn scroll_to(&self, reference: &str) -> Result<Done, BrowserError> {
        let marks = self.marks();
        let Aim { done, .. } = self.aim(reference).await?;
        self.settle(marks).await;
        Ok(done)
    }

    /// Goes back a page; `false` when there is none.
    pub async fn back(&self) -> Result<bool, BrowserError> {
        let (session, _) = self.page()?;
        let history = self
            .cdp
            .call(Some(&session), "Page.getNavigationHistory", json!({}))
            .await?;
        let current = history["currentIndex"].as_u64().unwrap_or_default() as usize;
        let Some(entry) = current
            .checked_sub(1)
            .and_then(|index| history["entries"].get(index))
        else {
            return Ok(false);
        };
        let marks = self.marks();
        self.cdp
            .call(
                Some(&session),
                "Page.navigateToHistoryEntry",
                json!({ "entryId": entry["id"] }),
            )
            .await?;
        self.settle(marks).await;
        Ok(true)
    }
}
