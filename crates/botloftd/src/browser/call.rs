//! One tool call's hold on a bot's browser: it waits for the call before
//! it, and starts the browser when the bot needs it (spec 21.2).

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BrowserState, BrowserStatus};
use tokio::sync::OwnedMutexGuard;
use tracing::debug;

use super::session::{Hooks, PageInfo, Session};
use super::{BrowserError, Browsers, launch, lock, update};

pub struct Call<'a> {
    pub(super) browsers: &'a Browsers,
    pub(super) bot: BotId,
    pub(super) _guard: OwnedMutexGuard<()>,
}

impl Call<'_> {
    /// The running browser, if there is one.
    pub fn running(&self) -> Option<Arc<Session>> {
        self.browsers.running(&self.bot)
    }

    /// The running browser, started if needed with downloads going to
    /// `downloads`.
    pub async fn session(&self, downloads: &Path) -> Result<Arc<Session>, BrowserError> {
        if let Some(session) = self.running() {
            return Ok(session);
        }
        let browsers = self.browsers;
        let bot = &self.bot;
        browsers.make_room(bot);
        browsers.set_state(bot, |state| {
            state.status = BrowserStatus::Starting;
            state.error = None;
        });
        let start = browsers.starts.fetch_add(1, Ordering::Relaxed);
        let started = match launch::find(&browsers.settings.path) {
            None => Err(BrowserError::NotFound),
            Some(program) => {
                let profile = browsers.profiles.join(bot.as_str());
                Session::start(&program, &profile, downloads, self.hooks(start)).await
            }
        };
        let session = match started {
            Ok(session) => session,
            Err(err) => {
                let error = err.to_string();
                browsers.set_state(bot, |state| {
                    state.status = BrowserStatus::Failed;
                    state.error = Some(error);
                });
                return Err(err);
            }
        };
        let watched = {
            let mut slots = lock(&browsers.slots);
            let slot = browsers.slot(&mut slots, bot);
            slot.session = Some((start, Arc::clone(&session)));
            slot.watchers > 0
        };
        browsers.set_state(bot, |state| state.status = BrowserStatus::Open);
        if watched {
            session.set_watching(true).await;
        }
        debug!(bot = %bot, "browser: open");
        Ok(session)
    }

    /// How the browser started as `start` reports back.
    fn hooks(&self, start: u64) -> Hooks {
        let browsers = self.browsers;
        let frames = {
            let mut slots = lock(&browsers.slots);
            browsers.slot(&mut slots, &self.bot).frames.clone()
        };
        let slots = Arc::clone(&browsers.slots);
        let events = browsers.events.clone();
        let clock = Arc::clone(&browsers.clock);
        let bot = self.bot.clone();
        let changed = {
            let (slots, events, clock, bot) = (
                Arc::clone(&slots),
                events.clone(),
                Arc::clone(&clock),
                bot.clone(),
            );
            move |info: PageInfo| {
                update(&slots, &events, clock.as_ref(), &bot, |state| {
                    state.url = info.url;
                    state.title = info.title;
                    state.loading = info.loading;
                    state.tabs = info.tabs;
                });
            }
        };
        let closed = move || {
            let current = match lock(&slots).get_mut(&bot) {
                Some(slot) if slot.session.as_ref().is_some_and(|(n, _)| *n == start) => {
                    slot.session = None;
                    slot.held.send_replace(None);
                    true
                }
                _ => false,
            };
            if current {
                update(&slots, &events, clock.as_ref(), &bot, |state| {
                    *state = BrowserState::closed(state.bot_id.clone(), state.updated_at);
                });
            }
        };
        Hooks {
            bot: self.bot.clone(),
            frames,
            changed: Box::new(changed),
            closed: Box::new(closed),
        }
    }
}
