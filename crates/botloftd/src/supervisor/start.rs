//! Starting a bot in three steps, so the supervisor lock is never held
//! while its files are written and its process is created (spec 7.4):
//! what to start is decided under the lock, the launch runs without it,
//! and the new process is recorded under the lock again.

use std::io;
use std::sync::Arc;

use botloft_core::ids::ConversationId;
use botloft_core::protocol::{AgentKind, BotState, Crew};
use botloft_store::BotRecord;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::slot::Running;
use super::spawn::{Launch, launch_spec, pump, settle};
use super::{ClaudeStatus, Inner, Supervisor};
use crate::agent::{self, ClaudeAgent};
use crate::chat::{StreamReader, control};
use crate::context;
use crate::runtime::Process;
use crate::runtime::claude::Claude;
use crate::seat::Seat;
use crate::state::Daemon;

/// A bot the reconcile pass decided to start.
pub(super) struct Start {
    crew: Crew,
    bot: BotRecord,
    /// The conversation of the bot this process holds (spec 40).
    conversation: ConversationId,
    /// Claude Code, for the bots that run on it; others start without.
    claude: Option<Claude>,
    /// The conversation to resume; `None` starts a new one.
    session: Option<String>,
}

impl Start {
    pub(super) fn bot_id(&self) -> &botloft_core::ids::BotId {
        &self.bot.id
    }

    pub(super) fn conversation(&self) -> &ConversationId {
        &self.conversation
    }
}

impl Supervisor {
    /// Decides to start `bot`, or marks it offline when Claude Code is not
    /// usable. The slot stays `launching` until [`Self::launch`] ends.
    pub(super) fn plan_start(
        &self,
        inner: &mut Inner,
        crew: &Crew,
        bot: &BotRecord,
        conversation: &ConversationId,
    ) -> Option<Start> {
        let Inner {
            slots,
            sessions,
            claude,
            ..
        } = inner;
        let slot = slots.get_mut(conversation)?;
        let claude = match claude {
            ClaudeStatus::Ready(claude) => Some(claude.clone()),
            // A bot on another agent does not need Claude Code (spec 30).
            _ if bot.agent != AgentKind::Claude => None,
            _ => {
                self.set_state(&bot.id, slot, BotState::Offline);
                return None;
            }
        };
        slot.launching = true;
        // This start reads the bot's launch settings.
        slot.restart_when_idle = false;
        let session = if slot.fresh_next {
            None
        } else {
            sessions.get(conversation).cloned()
        };
        Some(Start {
            crew: crew.clone(),
            bot: bot.clone(),
            conversation: conversation.clone(),
            claude,
            session,
        })
    }

    /// Writes the bot's files and creates its process without the
    /// supervisor lock, then records the process.
    pub(super) fn launch(&self, daemon: &Arc<Daemon>, start: Start) {
        let launched = launch_spec(
            daemon,
            &start.crew,
            &start.bot,
            start.claude.as_ref(),
            start.session.as_deref(),
        )
        .and_then(|(spec, launch)| Ok((self.runtime.spawn(spec)?, launch)));
        let mut inner = self.lock();
        self.commit(&mut inner, daemon, &start, launched);
    }

    fn commit(
        &self,
        inner: &mut Inner,
        daemon: &Arc<Daemon>,
        start: &Start,
        launched: io::Result<(Process, Launch)>,
    ) {
        let bot = &start.bot;
        let conversation = &start.conversation;
        let main = conversation.is_main_of(&bot.id);
        let Inner {
            slots,
            tokens,
            sessions,
            replaced,
            gone,
            ..
        } = inner;
        // Deleted while it started: the process goes with it.
        let slot = match slots.get_mut(conversation) {
            Some(slot) if !gone.contains(&bot.id) => slot,
            _ => {
                if let Ok((process, _)) = launched
                    && let Err(err) = process.control.kill()
                {
                    warn!(bot = %bot.id, "could not stop a deleted bot: {err}");
                }
                return;
            }
        };
        slot.launching = false;
        let (process, launch) = match launched {
            Ok(launched) => launched,
            Err(err) => {
                warn!(bot = %bot.id, "could not start the bot: {err}");
                slot.restart_at = Some(Instant::now() + slot.backoff.next_delay());
                self.set_state(&bot.id, slot, BotState::Backoff);
                return;
            }
        };
        let generation = self
            .next_generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // The agent of a launched bot runs; it may wrap the process to hold
        // a conversation with it (spec 30).
        let agent = agent::of(bot.agent).unwrap_or(&ClaudeAgent);
        let crate::runtime::Process {
            pid,
            events,
            control: raw,
        } = process;
        let (writer, decoder) = agent.attach(raw, launch.attach.clone());
        // What the session applies and how full it is (spec 9.2); Claude
        // Code answers as soon as it is up.
        let speaks = agent.speaks_control();
        if speaks {
            for ask in [control::settings_request(), control::context_request()] {
                let _ = writer.write(ask);
            }
        }
        // What is still the bot's, not each conversation's, follows the
        // main one (spec 40.9, C2).
        if main {
            // Each connected tool says whether it came up (spec 25.5).
            crate::service::mcp_state::forget(daemon, &bot.id);
            if launch.connected && speaks {
                let _ = writer.write(control::mcp_request());
            }
        }
        let seat = Seat::new(bot.id.clone(), conversation.clone());
        context::process_started(daemon, &seat, launch.resumed);
        // A new conversation is on disk only once it has a turn (spec
        // 7.3); the one it takes the place of is left behind.
        if !launch.resumed
            && let Some(old) = sessions.remove(conversation)
        {
            replaced.push((bot.id.clone(), conversation.clone(), old));
        }
        slot.generation = Some(generation);
        slot.restart_at = None;
        slot.fresh_next = false;
        slot.clear_work();
        slot.limited_until = None;
        tokens.insert(launch.token_hash.clone(), (seat.clone(), generation));
        slot.running = Some(Running {
            control: writer,
            started: Instant::now(),
            resumed: launch.resumed,
            token_hash: launch.token_hash,
            permission_mode: bot.permission_mode,
            effort: bot.effort,
        });
        self.count_busy(slot.state, BotState::Launching);
        slot.state = BotState::Launching;
        self.announce(&bot.id, slot);
        debug!(bot = %bot.id, generation, pid = ?pid, resumed = launch.resumed, "bot started");
        let reader = StreamReader::new(seat, generation, decoder);
        tokio::spawn(pump(self.daemon.clone(), reader, events));
        tokio::spawn(settle(
            self.daemon.clone(),
            bot.id.clone(),
            generation,
            self.settings.ready_after,
        ));
        // A launch setting changed while it started (spec 7.4).
        self.relaunch_if_idle(&bot.id, slot);
    }
}
