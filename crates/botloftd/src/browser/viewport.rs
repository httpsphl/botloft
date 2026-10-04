//! The size of the page (spec 21.3): a desktop screen while the bot works
//! alone, and the shape and width of the owner's panel while they watch,
//! so the page shows near its own size there, drawn as sharp as their
//! screen. The bot's own pictures stay the size of the page.

use serde_json::{Value, json};
use tracing::debug;

use super::BrowserError;
use super::session::{FRAME_GAP, Session};

/// The page size bots and the owner see, in CSS pixels. The height is the
/// one nobody asked to change.
pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 800;
/// How narrow and how wide the owner's panel may make the page: narrower,
/// most sites would drop their desktop layout.
pub const MIN_WIDTH: u32 = 800;
pub const MAX_WIDTH: u32 = 1600;
/// How short and how tall the owner's panel may make the page.
pub const MIN_HEIGHT: u32 = 600;
pub const MAX_HEIGHT: u32 = 2000;
/// The sharpest the page is drawn for the owner, in percent: two pixels of
/// their screen to one of the page.
pub const MAX_SCALE: u32 = 200;

/// The size of the page, the same in every tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    /// The owner's screen pixels per pixel of the page, in percent.
    pub scale: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
            scale: 100,
        }
    }
}

impl Viewport {
    /// The page for a panel with `width` by `height` of room: as wide as
    /// the room, so it shows at its own size, within limits that keep the
    /// desktop layout, and as tall as the room's shape asks.
    pub fn fitting(width: u32, height: u32) -> Self {
        let wide = width.clamp(MIN_WIDTH, MAX_WIDTH);
        let tall = u64::from(wide) * u64::from(height) / u64::from(width.max(1));
        Self {
            width: wide,
            height: u32::try_from(tall)
                .unwrap_or(MAX_HEIGHT)
                .clamp(MIN_HEIGHT, MAX_HEIGHT),
            scale: 100,
        }
    }

    /// The same page, drawn `scale` percent as sharp, within limits.
    pub fn at_scale(self, scale: u32) -> Self {
        Self {
            scale: scale.clamp(100, MAX_SCALE),
            ..self
        }
    }

    fn factor(self) -> f64 {
        f64::from(self.scale) / 100.0
    }

    pub(super) fn metrics(self) -> Value {
        json!({
            "width": self.width, "height": self.height, "deviceScaleFactor": self.factor(),
            "mobile": false,
        })
    }

    /// Whether a point is on the page.
    pub fn contains(self, x: f64, y: f64) -> bool {
        (0.0..=f64::from(self.width)).contains(&x) && (0.0..=f64::from(self.height)).contains(&y)
    }
}

impl Session {
    pub fn viewport(&self) -> Viewport {
        self.lock().viewport
    }

    /// Makes the page `viewport` in size, in every tab, and sends whoever
    /// watches a frame of that size.
    pub async fn set_viewport(&self, viewport: Viewport) {
        let pages: Vec<String> = {
            let mut tabs = self.lock();
            if tabs.viewport == viewport {
                return;
            }
            tabs.viewport = viewport;
            tabs.list.iter().map(|tab| tab.session.clone()).collect()
        };
        for page in pages {
            let sized = self
                .cdp
                .call(
                    Some(&page),
                    "Emulation.setDeviceMetricsOverride",
                    viewport.metrics(),
                )
                .await;
            if let Err(err) = sized {
                debug!("browser: sizing a tab: {err}");
            }
        }
        // The browser drops the frame of the new size while older ones wait
        // to be confirmed, and a page that stands still sends no other
        // (spec 19): once those are confirmed, the frames start over.
        tokio::time::sleep(FRAME_GAP * 2).await;
        let watched = {
            let tabs = self.lock();
            tabs.active()
                .filter(|_| tabs.watching)
                .map(|tab| tab.session.clone())
        };
        if let Some(page) = watched {
            self.cast(&page, false).await;
            self.cast(&page, true).await;
        }
    }

    /// The screen as a JPEG, base64, for the bot: the size of the page,
    /// however sharp the owner's screen has it drawn.
    pub async fn screenshot(&self) -> Result<String, BrowserError> {
        self.capture(false).await
    }

    /// The screen as a JPEG, base64, as sharp as the owner's panel draws it.
    pub async fn picture(&self) -> Result<String, BrowserError> {
        self.capture(true).await
    }

    async fn capture(&self, sharp: bool) -> Result<String, BrowserError> {
        let (session, _) = self.page()?;
        let page = self.viewport();
        let mut params = json!({ "format": "jpeg", "quality": 70 });
        if !sharp && page.scale > 100 {
            // The page where it is scrolled to, scaled back down (spec 19).
            let metrics = self
                .cdp
                .call(Some(&session), "Page.getLayoutMetrics", json!({}))
                .await?;
            let view = &metrics["cssVisualViewport"];
            params["clip"] = json!({
                "x": view["pageX"], "y": view["pageY"],
                "width": page.width, "height": page.height,
                "scale": 1.0 / page.factor(),
            });
        }
        let shot = self
            .cdp
            .call(Some(&session), "Page.captureScreenshot", params)
            .await?;
        shot["data"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| BrowserError::Page("the browser returned no picture".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_takes_the_shape_of_the_room_within_limits() {
        let page = |width, height| Viewport::fitting(width, height);
        // A panel 1000 wide with 700 of height: the same size.
        assert_eq!(
            page(1000, 700),
            Viewport {
                width: 1000,
                height: 700,
                scale: 100,
            }
        );
        // A narrow one keeps the desktop layout, in the room's shape.
        assert_eq!(
            page(536, 700),
            Viewport {
                width: 800,
                height: 1044,
                scale: 100,
            }
        );
        assert_eq!(page(1280, 800), Viewport::default());
        assert_eq!(page(2400, 900).width, MAX_WIDTH);
        // Too flat or too tall a room stops at the limits.
        assert_eq!(page(1600, 300).height, MIN_HEIGHT);
        assert_eq!(page(256, 1400).height, MAX_HEIGHT);
        assert_eq!(page(0, 0).height, MIN_HEIGHT);
        assert_eq!(page(1, u32::MAX).height, MAX_HEIGHT);
    }

    #[test]
    fn the_page_is_drawn_as_sharp_as_the_screen_up_to_twice() {
        let page = Viewport::fitting(1000, 700);
        assert_eq!(page.scale, 100);
        assert_eq!(page.at_scale(150).scale, 150);
        assert_eq!(page.at_scale(300).scale, MAX_SCALE);
        // A screen less sharp than the page still gets one pixel each.
        assert_eq!(page.at_scale(80).scale, 100);
        assert_eq!(page.at_scale(150).metrics()["deviceScaleFactor"], 1.5);
        assert_eq!(page.at_scale(150).width, 1000);
    }

    #[test]
    fn points_are_on_the_page_up_to_its_edges() {
        let page = Viewport::fitting(640, 700);
        assert!(page.contains(800.0, 875.0));
        assert!(!page.contains(10.0, 875.5));
        assert!(!page.contains(800.5, 10.0));
        assert!(!page.contains(-1.0, 10.0));
        assert!(!page.contains(f64::NAN, 10.0));
    }
}
