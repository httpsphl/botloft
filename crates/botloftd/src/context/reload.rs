//! How much of a turn's cache write was the conversation sent again (spec
//! 8.7). Each model request sends the whole conversation; what the prompt
//! cache still had comes back as `cache_read_input_tokens`. When the cache
//! expired, because the bot sat idle or its process restarted, the part of
//! the conversation it no longer had is written again and counts as
//! `cache_creation_input_tokens`, next to what is really new.

use serde_json::Value;

#[derive(Default)]
pub(super) struct Tracker {
    /// What the conversation's last model request sent, or the size Claude
    /// Code told before any request was seen. `None` for a conversation
    /// that has not been measured, such as one that just began.
    sent: Option<u64>,
    /// The message of the last request seen: one request can print several
    /// `assistant` lines with the same usage.
    message: Option<String>,
    /// Summed over the turn in progress.
    turn: u64,
}

impl Tracker {
    /// One model request of the conversation, with its `usage`.
    pub(super) fn request(&mut self, message: Option<&str>, usage: &Value) {
        let count = |key: &str| usage[key].as_u64().unwrap_or_default();
        let (input, written, read) = (
            count("input_tokens"),
            count("cache_creation_input_tokens"),
            count("cache_read_input_tokens"),
        );
        if message.is_some() && message == self.message.as_deref() {
            return;
        }
        self.message = message.map(str::to_owned);
        if let Some(before) = self.sent {
            // What was already there and did not come from the cache.
            self.turn += before.saturating_sub(read).min(written);
        }
        self.sent = Some(input + written + read);
    }

    /// Claude Code told how big the conversation is. A request knows better,
    /// so this only fills in for a conversation with none seen yet.
    pub(super) fn measured(&mut self, size: u64) {
        self.sent.get_or_insert(size);
    }

    /// The conversation began again, as a new one or as a summary.
    pub(super) fn restart(&mut self) {
        self.sent = None;
        self.message = None;
    }

    /// The turn ended: what it wrote again, and a new count for the next.
    pub(super) fn take(&mut self) -> u64 {
        std::mem::take(&mut self.turn)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn usage(input: u64, written: u64, read: u64) -> Value {
        json!({
            "input_tokens": input,
            "cache_creation_input_tokens": written,
            "cache_read_input_tokens": read,
            "output_tokens": 50,
        })
    }

    #[test]
    fn a_warm_cache_writes_nothing_again() {
        let mut tracker = Tracker::default();
        tracker.measured(40_000);
        tracker.request(Some("msg_1"), &usage(2, 600, 40_000));
        tracker.request(Some("msg_2"), &usage(2, 3_000, 40_602));
        assert_eq!(tracker.take(), 0);
    }

    #[test]
    fn an_expired_cache_writes_the_conversation_again() {
        let mut tracker = Tracker::default();
        tracker.measured(72_000);
        // Only Claude Code's own instructions were still cached.
        tracker.request(Some("msg_1"), &usage(2, 52_600, 20_000));
        assert_eq!(tracker.take(), 52_000, "the new 600 are not counted");
        // Nothing carries over to the next turn.
        tracker.request(Some("msg_2"), &usage(2, 400, 72_602));
        assert_eq!(tracker.take(), 0);
    }

    #[test]
    fn a_request_counts_once_whatever_lines_it_prints() {
        let mut tracker = Tracker::default();
        tracker.measured(30_000);
        let cold = usage(2, 30_500, 0);
        tracker.request(Some("msg_1"), &cold);
        tracker.request(Some("msg_1"), &cold);
        assert_eq!(tracker.take(), 30_000);
    }

    #[test]
    fn a_conversation_not_measured_counts_nothing_again() {
        let mut tracker = Tracker::default();
        tracker.request(Some("msg_1"), &usage(3, 21_000, 0));
        assert_eq!(tracker.take(), 0, "a new conversation is all new");
        tracker.restart();
        tracker.request(Some("msg_2"), &usage(3, 9_000, 0));
        assert_eq!(tracker.take(), 0, "a summary is new too");
    }

    #[test]
    fn a_size_told_later_does_not_replace_a_request() {
        let mut tracker = Tracker::default();
        tracker.request(Some("msg_1"), &usage(2, 1_000, 40_000));
        // Told at the end of the turn, with the reply in it.
        tracker.measured(41_900);
        tracker.request(Some("msg_2"), &usage(2, 1_500, 41_002));
        assert_eq!(tracker.take(), 0);
    }
}
