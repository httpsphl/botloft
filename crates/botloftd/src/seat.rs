//! A bot in one of its conversations (spec 40): what a process speaks for.
//! It reads as the bot wherever only the bot matters, so the code that
//! follows a process passes it on as it passed the bot.

use std::fmt;
use std::ops::Deref;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::BotState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seat {
    pub bot: BotId,
    pub conversation: ConversationId,
}

impl Seat {
    pub fn new(bot: BotId, conversation: ConversationId) -> Self {
        Self { bot, conversation }
    }

    /// The main conversation of `bot`.
    pub fn main(bot: &BotId) -> Self {
        Self::new(bot.clone(), ConversationId::main_of(bot))
    }

    pub fn is_main(&self) -> bool {
        self.conversation.is_main_of(&self.bot)
    }
}

impl Deref for Seat {
    type Target = BotId;

    fn deref(&self) -> &BotId {
        &self.bot
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.bot, f)
    }
}

/// One conversation's process changed state (spec 40.4): the courier
/// writes to it once it is ready, and the app hears of it as
/// `conversation.changed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatState {
    pub seat: Seat,
    pub state: BotState,
    pub generation: Option<u64>,
}
