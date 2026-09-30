//! A bot's chat (spec 8): reads the stream-json its process prints, line by
//! line, and turns each event into chat items, live text, read receipts and
//! state changes.

mod account;
pub(crate) mod control;
mod events;
pub(crate) mod items;
mod session;

use botloft_core::ids::BotId;
use tracing::debug;

use crate::state::Daemon;

/// Longest line kept; anything longer is dropped whole (spec 8.1).
const LINE_MAX: usize = 8 << 20;

/// Splits one process's stdout into lines. Lines may arrive split across
/// chunks, and one chunk may hold several.
pub struct StreamReader {
    bot: BotId,
    generation: u64,
    buffer: Vec<u8>,
    /// Inside a line that grew past [`LINE_MAX`]; ends at its newline.
    skipping: bool,
}

impl StreamReader {
    pub fn new(bot: BotId, generation: u64) -> Self {
        Self {
            bot,
            generation,
            buffer: Vec::new(),
            skipping: false,
        }
    }

    pub fn bot(&self) -> &BotId {
        &self.bot
    }

    pub fn generation(&self) -> u64 {
        self.generation
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

    fn line(&self, daemon: &Daemon) {
        let line = self.buffer.strip_suffix(b"\r").unwrap_or(&self.buffer);
        if line.iter().all(u8::is_ascii_whitespace) {
            return;
        }
        match serde_json::from_slice(line) {
            Ok(event) => events::apply(daemon, &self.bot, self.generation, &event),
            Err(_) => debug!(bot = %self.bot, "stream line is not JSON"),
        }
    }
}
