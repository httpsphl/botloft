//! The size of the page (spec 21.3): as wide as a desktop screen always,
//! and as tall as the owner's panel asks while they watch.

use serde_json::{Value, json};
use tracing::debug;

use super::session::Session;

/// The page size bots and the owner see, in CSS pixels. The height is the
/// one nobody asked to change.
pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 800;
/// How short and how tall the owner's panel may make the page.
pub const MIN_HEIGHT: u32 = 600;
pub const MAX_HEIGHT: u32 = 2000;

/// The size of the page, the same in every tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
        }
    }
}

impl Viewport {
    /// The page for a panel with `width` by `height` of room: as wide as
    /// ever, so sites keep their desktop layout, and as tall as the room's
    /// shape asks, within limits.
    pub fn fitting(width: u32, height: u32) -> Self {
        let tall = u64::from(WIDTH) * u64::from(height) / u64::from(width.max(1));
        Self {
            width: WIDTH,
            height: u32::try_from(tall)
                .unwrap_or(MAX_HEIGHT)
                .clamp(MIN_HEIGHT, MAX_HEIGHT),
        }
    }

    pub(super) fn metrics(self) -> Value {
        json!({
            "width": self.width, "height": self.height, "deviceScaleFactor": 1, "mobile": false,
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

    /// Makes the page `viewport` in size, in every tab.
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_takes_the_shape_of_the_room_within_limits() {
        let page = |width, height| Viewport::fitting(width, height);
        // A panel 536 wide with 700 of height: the same shape, 1280 wide.
        assert_eq!(
            page(536, 700),
            Viewport {
                width: 1280,
                height: 1671
            }
        );
        assert_eq!(page(1280, 800), Viewport::default());
        // Too flat or too tall a room stops at the limits.
        assert_eq!(page(1600, 300).height, MIN_HEIGHT);
        assert_eq!(page(256, 1400).height, MAX_HEIGHT);
        assert_eq!(page(0, 0).height, MIN_HEIGHT);
        assert_eq!(page(1, u32::MAX).height, MAX_HEIGHT);
    }

    #[test]
    fn points_are_on_the_page_up_to_its_edges() {
        let page = Viewport::fitting(640, 700);
        assert!(page.contains(1280.0, 1400.0));
        assert!(!page.contains(10.0, 1400.5));
        assert!(!page.contains(-1.0, 10.0));
        assert!(!page.contains(f64::NAN, 10.0));
    }
}
