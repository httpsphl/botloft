//! The live picture of the window a bot is using, for whoever watches its
//! desktop panel (spec 24.9). While someone watches, the window is
//! pictured four times a second and a picture goes out only when it
//! changed; when the last one stops, so does the picturing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::DesktopFrame;
use tokio::sync::watch;

use super::activity::Activity;
use crate::platform::desktop;

/// How often a watched window is pictured.
const EVERY: Duration = Duration::from_millis(250);

type Frames = watch::Sender<Option<Arc<DesktopFrame>>>;

/// Held by the connection; dropping it stops watching.
pub struct DesktopWatching {
    pub bot: BotId,
    /// The newest picture; `changed()` wakes on each new one.
    pub frames: watch::Receiver<Option<Arc<DesktopFrame>>>,
}

#[derive(Default)]
pub struct Screens {
    /// Each watched bot's pictures, while its picturing runs.
    running: Mutex<HashMap<BotId, Frames>>,
}

impl Screens {
    /// Starts watching the bot's window: its picturing runs while anyone
    /// watches.
    pub fn watch(self: &Arc<Self>, bot: &BotId, activity: &Arc<Activity>) -> DesktopWatching {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(frames) = running.get(bot) {
            return DesktopWatching {
                bot: bot.clone(),
                frames: frames.subscribe(),
            };
        }
        let (frames, watching) = watch::channel(None);
        running.insert(bot.clone(), frames.clone());
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let (screens, activity, bot) = (Arc::clone(self), Arc::clone(activity), bot.clone());
            runtime.spawn(async move { screens.picture(&bot, &activity, &frames).await });
        }
        DesktopWatching {
            bot: bot.clone(),
            frames: watching,
        }
    }

    /// The newest picture of the bot's window, while it is watched.
    pub fn now(&self, bot: &BotId) -> Option<DesktopFrame> {
        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(bot)
            .and_then(|frames| frames.borrow().as_deref().cloned())
    }

    /// Pictures the bot's window until nobody watches.
    async fn picture(&self, bot: &BotId, activity: &Activity, frames: &Frames) {
        let mut last: Option<String> = None;
        loop {
            {
                let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
                if frames.receiver_count() == 0 {
                    running.remove(bot);
                    return;
                }
            }
            if let Some(id) = activity.window(bot) {
                let picture = tokio::task::spawn_blocking(move || desktop::picture(id)).await;
                if let Ok(Ok(picture)) = picture
                    && !picture.jpeg.is_empty()
                {
                    let data = BASE64.encode(&picture.jpeg);
                    if last.as_deref() != Some(data.as_str()) {
                        last = Some(data.clone());
                        frames.send_replace(Some(Arc::new(DesktopFrame {
                            bot_id: bot.clone(),
                            data,
                            width: picture.width,
                            height: picture.height,
                        })));
                    }
                }
            }
            tokio::time::sleep(EVERY).await;
        }
    }
}
