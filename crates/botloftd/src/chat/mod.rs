//! A bot's chat (spec 8): reads the stream-json its process prints, line by
//! line, and turns each event into chat items, live text, read receipts and
//! state changes.

mod account;
pub(crate) mod control;
mod events;
pub(crate) mod items;
mod session;

use botloft_core::ids::BotId;
use botloft_core::protocol::ChatDelta;
use tracing::debug;

use crate::state::{Daemon, Event};

/// Longest line kept; anything longer is dropped whole (spec 8.1).
const LINE_MAX: usize = 8 << 20;

/// Live text waits at most this long to go out with the pieces after it
/// (spec 8.3): one `chat.delta` per token cost every app a render.
pub const LIVE_TEXT_EVERY: std::time::Duration = std::time::Duration::from_millis(40);

/// Live text this long goes out without waiting.
const LIVE_TEXT_MAX: usize = 4 << 10;

/// Splits one process's stdout into lines. Lines may arrive split across
/// chunks, and one chunk may hold several.
pub struct StreamReader {
    bot: BotId,
    generation: u64,
    buffer: Vec<u8>,
    /// Inside a line that grew past [`LINE_MAX`]; ends at its newline.
    skipping: bool,
    /// Live text not sent yet.
    live: String,
}

impl StreamReader {
    pub fn new(bot: BotId, generation: u64) -> Self {
        Self {
            bot,
            generation,
            buffer: Vec::new(),
            skipping: false,
            live: String::new(),
        }
    }

    pub fn bot(&self) -> &BotId {
        &self.bot
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// True while live text waits for [`StreamReader::flush`].
    pub fn holds_live_text(&self) -> bool {
        !self.live.is_empty()
    }

    /// Sends the live text gathered so far as one `chat.delta`.
    pub fn flush(&mut self, daemon: &Daemon) {
        if !self.live.is_empty() {
            daemon.emit(Event::ChatDelta(ChatDelta {
                bot_id: self.bot.clone(),
                text: std::mem::take(&mut self.live),
            }));
        }
    }

    pub fn feed(&mut self, daemon: &Daemon, data: &[u8]) {
        let mut rest = data;
        while let Some(end) = rest.iter().position(|byte| *byte == b'\n') {
            if self.skipping {
                self.skipping = false;
            } else {
                self.buffer.extend_from_slice(&rest[..end]);
                self.line(daemon);
            }
            self.buffer.clear();
            rest = &rest[end + 1..];
        }
        if self.skipping {
            return;
        }
        self.buffer.extend_from_slice(rest);
        if self.buffer.len() > LINE_MAX {
            debug!(bot = %self.bot, "dropped a stream line longer than 8 MiB");
            self.buffer.clear();
            self.skipping = true;
        }
    }

    fn line(&mut self, daemon: &Daemon) {
        let line = self.buffer.strip_suffix(b"\r").unwrap_or(&self.buffer);
        if line.iter().all(u8::is_ascii_whitespace) {
            return;
        }
        let event = match serde_json::from_slice(line) {
            Ok(event) => event,
            Err(_) => {
                debug!(bot = %self.bot, "stream line is not JSON");
                return;
            }
        };
        if let Some(text) = events::live_text(&event) {
            // Lines from a process that was replaced say nothing about the
            // new one.
            if daemon.supervisor.is_current(&self.bot, self.generation) {
                self.live.push_str(text);
                if self.live.len() >= LIVE_TEXT_MAX {
                    self.flush(daemon);
                }
            }
            return;
        }
        // Whatever comes next follows the text before it.
        self.flush(daemon);
        events::apply(daemon, &self.bot, self.generation, &event);
    }
}
