//! The conversations on the phone (spec 28.12): which one each phone has
//! open, the reply being written, and the pace of what is sent about it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    ApprovalStatus, ChatBody, ChatDelta, FromPhone, MessagesSendParams, PhoneItem, ToPhone,
};

use super::Mobile;
use super::bridge::blocking_result;
use super::chats;
use crate::service::{ApiError, messages};
use crate::state::Daemon;

/// The least time between two `live` messages to a phone.
const LIVE_EVERY_MS: i64 = 1000;
/// The least time between two `line` messages about one bot.
const LINE_EVERY_MS: i64 = 2000;
/// How much of a reply being written is kept; the phone gets its end.
const LIVE_KEEP: usize = 64 * 1024;

#[derive(Default)]
pub(crate) struct Talk {
    /// The bot each phone has open.
    watching: HashMap<String, BotId>,
    /// The reply each bot is writing now, whole.
    live: HashMap<BotId, String>,
    /// When each phone last got a `live`.
    live_sent: HashMap<String, i64>,
    /// When a `line` about each bot last went out.
    line_sent: HashMap<BotId, i64>,
}

impl Talk {
    fn lock(this: &Mutex<Self>) -> MutexGuard<'_, Self> {
        this.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Why a send did not happen, in the words the phone knows.
fn reason(error: &ApiError) -> &'static str {
    match error {
        ApiError::Validation(_) => "invalid",
        ApiError::NotFound(_) | ApiError::Conflict(_) => "not_found",
        _ => "failed",
    }
}

impl Mobile {
    /// A phone left: what it had open is forgotten.
    pub(crate) fn leave(&self, phone: &str) {
        let mut talk = Talk::lock(&self.talk);
        talk.watching.remove(phone);
        talk.live_sent.remove(phone);
    }

    pub(crate) fn leave_all(&self) {
        let mut talk = Talk::lock(&self.talk);
        talk.watching.clear();
        talk.live_sent.clear();
    }

    fn watchers(&self, daemon: &Daemon, bot: &BotId) -> Vec<String> {
        let online = self.online_phones(daemon);
        let talk = Talk::lock(&self.talk);
        online
            .into_iter()
            .filter(|phone| talk.watching.get(&phone.id) == Some(bot))
            .map(|phone| phone.id)
            .collect()
    }

    /// An item of a chat is new or changed.
    pub(crate) fn chat_item_changed(
        &self,
        daemon: &Daemon,
        item: &botloft_core::protocol::ChatItem,
    ) {
        let Some(shown) = chats::item(daemon, item) else {
            return;
        };
        if matches!(item.body, ChatBody::Reply(_)) {
            // The reply is whole now: what was being written is done.
            let had = Talk::lock(&self.talk).live.remove(&item.bot_id).is_some();
            if had {
                for phone in self.watchers(daemon, &item.bot_id) {
                    self.send_to(
                        daemon,
                        &phone,
                        &ToPhone::Live {
                            bot_id: item.bot_id.clone(),
                            text: String::new(),
                        },
                    );
                }
            }
        }
        for phone in self.watchers(daemon, &item.bot_id) {
            self.send_to(
                daemon,
                &phone,
                &ToPhone::Item {
                    bot_id: item.bot_id.clone(),
                    item: shown.clone(),
                },
            );
        }
        // Only what the owner wants to hear about moves the list.
        let worth = match &shown {
            PhoneItem::Reply { .. } | PhoneItem::Question { .. } | PhoneItem::Notice { .. } => true,
            PhoneItem::Approval { status, .. } => *status == ApprovalStatus::Pending,
            _ => false,
        };
        let now = daemon.clock.now_ms();
        let due = worth && {
            let mut talk = Talk::lock(&self.talk);
            let last = talk
                .line_sent
                .get(&item.bot_id)
                .copied()
                .unwrap_or(i64::MIN / 2);
            let due = now - last >= LINE_EVERY_MS;
            if due {
                talk.line_sent.insert(item.bot_id.clone(), now);
            }
            due
        };
        if due && let Some(bot) = chats::line(daemon, &item.bot_id) {
            for phone in self.online_phones(daemon) {
                self.send_to(daemon, &phone.id, &ToPhone::Line { bot: bot.clone() });
            }
        }
    }

    /// More of the reply a bot is writing.
    pub(crate) fn chat_delta(&self, daemon: &Daemon, delta: &ChatDelta) {
        let text = {
            let mut talk = Talk::lock(&self.talk);
            let live = talk.live.entry(delta.bot_id.clone()).or_default();
            live.push_str(&delta.text);
            if live.len() > LIVE_KEEP {
                let mut start = live.len() - LIVE_KEEP;
                while !live.is_char_boundary(start) {
                    start += 1;
                }
                live.drain(..start);
            }
            chats::tail(live)
        };
        let now = daemon.clock.now_ms();
        for phone in self.watchers(daemon, &delta.bot_id) {
            let due = {
                let mut talk = Talk::lock(&self.talk);
                let last = talk.live_sent.get(&phone).copied().unwrap_or(i64::MIN / 2);
                let due = now - last >= LIVE_EVERY_MS;
                if due {
                    talk.live_sent.insert(phone.clone(), now);
                }
                due
            };
            if due {
                self.send_to(
                    daemon,
                    &phone,
                    &ToPhone::Live {
                        bot_id: delta.bot_id.clone(),
                        text: text.clone(),
                    },
                );
            }
        }
    }

    /// A bot started or stopped working.
    pub(crate) fn bot_state(
        &self,
        daemon: &Daemon,
        bot: &BotId,
        state: botloft_core::protocol::BotState,
    ) {
        use botloft_core::protocol::BotState;
        if !matches!(state, BotState::Busy | BotState::NeedsApproval)
            && Talk::lock(&self.talk).live.remove(bot).is_some()
        {
            for phone in self.watchers(daemon, bot) {
                self.send_to(
                    daemon,
                    &phone,
                    &ToPhone::Live {
                        bot_id: bot.clone(),
                        text: String::new(),
                    },
                );
            }
        }
        for phone in self.watchers(daemon, bot) {
            self.send_to(
                daemon,
                &phone,
                &ToPhone::State {
                    bot_id: bot.clone(),
                    state,
                },
            );
        }
    }

    /// The messages about conversations that a phone sends.
    pub(crate) async fn talk_apply(&self, daemon: &Arc<Daemon>, phone: &str, message: FromPhone) {
        match message {
            FromPhone::Chats => {
                let parts =
                    blocking_result(daemon, |daemon| chats::chats_parts(chats::lines(daemon)))
                        .await;
                for part in parts.unwrap_or_default() {
                    self.send_to(daemon, phone, &part);
                }
            }
            FromPhone::History {
                req,
                bot_id,
                before,
            } => {
                let id = bot_id.clone();
                let parts = blocking_result(daemon, move |daemon| {
                    let (items, more) =
                        chats::page(daemon, &id, before.as_ref()).unwrap_or_default();
                    chats::history_parts(req, &id, items, more)
                })
                .await;
                for part in parts.unwrap_or_default() {
                    self.send_to(daemon, phone, &part);
                }
            }
            FromPhone::Watch { bot_id } => self.watch(daemon, phone, bot_id),
            FromPhone::Send {
                client_id,
                bot_id,
                text,
            } => {
                let result = if self.may_send(daemon, phone) {
                    blocking_result(daemon, move |daemon| {
                        messages::send(
                            daemon,
                            MessagesSendParams {
                                bot_id,
                                body: text,
                                attachments: None,
                                reply_to: None,
                            },
                        )
                        .map(|_| ())
                        .map_err(|error| reason(&error))
                    })
                    .await
                    .unwrap_or(Err("failed"))
                } else {
                    Err("rate_limited")
                };
                self.send_to(
                    daemon,
                    phone,
                    &ToPhone::Sent {
                        client_id,
                        ok: result.is_ok(),
                        reason: result.err().map(str::to_owned),
                    },
                );
            }
            _ => {}
        }
    }

    /// Which conversation the phone has open; what it needs to follow it is
    /// sent at once.
    fn watch(&self, daemon: &Daemon, phone: &str, bot: Option<BotId>) {
        {
            let mut talk = Talk::lock(&self.talk);
            talk.live_sent.remove(phone);
            match &bot {
                Some(bot) => talk.watching.insert(phone.to_owned(), bot.clone()),
                None => talk.watching.remove(phone),
            };
        }
        let Some(bot) = bot else {
            return;
        };
        let Some(line) = chats::line(daemon, &bot) else {
            return;
        };
        self.send_to(
            daemon,
            phone,
            &ToPhone::State {
                bot_id: bot.clone(),
                state: line.state,
            },
        );
        let live = Talk::lock(&self.talk).live.get(&bot).cloned();
        if let Some(live) = live {
            self.send_to(
                daemon,
                phone,
                &ToPhone::Live {
                    bot_id: bot,
                    text: chats::tail(&live),
                },
            );
        }
    }
}
