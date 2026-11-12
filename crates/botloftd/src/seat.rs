//! A bot in one of its conversations (spec 40): what a process speaks for.
//! It reads as the bot wherever only the bot matters, so the code that
//! follows a process passes it on as it passed the bot.

use std::fmt;
use std::ops::Deref;

use botloft_core::ids::{BotId, ConversationId};

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
