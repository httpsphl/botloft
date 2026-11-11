//! A card a bot shows lives in its chat item (spec 8.11). The owner's
//! answer rewrites the item and saves the message that carries it, in one
//! transaction.

use botloft_core::ids::ChatItemId;
use botloft_core::protocol::{ChatBody, ChatItem, Delivery, Message};

use crate::{Result, Store};

impl Store {
    /// Saves the card `id` as `body`, now with the owner's answer, and
    /// `message` (the answer, to the bot) with its delivery. `None`, with
    /// nothing saved, if there is no such item. Returns the card and the
    /// message's own item.
    pub fn answer_card(
        &self,
        id: &ChatItemId,
        body: &ChatBody,
        message: &Message,
        delivery: &Delivery,
    ) -> Result<Option<(ChatItem, ChatItem)>> {
        let tx = self.conn.unchecked_transaction()?;
        let Some(card) = Self::update_chat_item_in(&tx, id, body, message.created_at)? else {
            return Ok(None);
        };
        let answer = Self::insert_message_in(&tx, message, delivery)?;
        tx.commit()?;
        Ok(Some((card, answer)))
    }
}
