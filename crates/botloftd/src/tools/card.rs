//! `show_card` (spec 8.11): the bot shows the owner a card in its chat,
//! with text, tables, charts, buttons or a form, or shows one again,
//! changed. The owner's answer comes back later as their message.

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{
    CARD_BLOCKS_MAX, CARD_BUTTONS_MAX, CARD_FIELDS_MAX, CARD_TITLE_MAX_CHARS, CardBlock,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::service::cards::{self, Show};
use crate::state::Daemon;

pub const SHOW_CARD: &str = "show_card";

pub(super) fn tool() -> Value {
    let text = json!({ "type": "string" });
    json!({
        "name": SHOW_CARD,
        "title": "Show a card",
        "description": format!(
            "Shows the owner a card in your chat: text, tables, bar or line charts, buttons \
             and a form. Use it when a table, a chart or a choice says it better than prose, \
             e.g. a weekly report, or options to pick from. To change a card you showed, \
             call it again with `card` and the whole new content: it changes in place, as a \
             new version. When the owner presses a button or sends the form, it arrives \
             later as their message that starts with \"Answer on your card <id>\"; each \
             version takes one answer, so show it again to ask again. Don't wait for the \
             answer: carry on or end your turn. At most {CARD_BLOCKS_MAX} blocks, \
             {CARD_BUTTONS_MAX} buttons and one form of {CARD_FIELDS_MAX} fields."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "card": {
                    "type": "string",
                    "description": "The card_id of a card you showed, to show it again \
                        changed. Leave it out for a new card.",
                },
                "title": {
                    "type": "string",
                    "maxLength": CARD_TITLE_MAX_CHARS,
                    "description": "One line, in the language the owner writes to you in.",
                },
                "blocks": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": CARD_BLOCKS_MAX,
                    "description": "The card's parts, top to bottom. Each has a `type`: \
                        text {text} (markdown); table {columns, rows} (rows of cells, one per \
                        column); chart {kind: bar|line, labels, series: [{name, values}], \
                        unit?} (one value per label, null for none); buttons {buttons: \
                        [{id, label}]}; form {fields: [{id, label, kind: \
                        text|long_text|number|choice|check, options? (for a choice), \
                        required?}], submit?}. Ids are short: letters, digits, - and _.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["text", "table", "chart", "buttons", "form"],
                            },
                            "text": text,
                            "columns": { "type": "array", "items": text },
                            "rows": { "type": "array", "items": { "type": "array" } },
                            "kind": { "type": "string" },
                            "labels": { "type": "array", "items": text },
                            "series": { "type": "array", "items": { "type": "object" } },
                            "unit": text,
                            "buttons": { "type": "array", "items": { "type": "object" } },
                            "fields": { "type": "array", "items": { "type": "object" } },
                            "submit": text,
                        },
                        "required": ["type"],
                    },
                },
            },
            "required": ["title", "blocks"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShowArgs {
    #[serde(default)]
    card: Option<String>,
    title: String,
    blocks: Vec<Value>,
}

pub(super) fn show(daemon: &Daemon, bot: &BotId, args: ShowArgs) -> Result<Value, String> {
    let card =
        match args.card.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(id) => Some(id.parse::<ChatItemId>().map_err(|_| {
                format!("card must be the card_id show_card gave you, not \"{id}\"")
            })?),
        };
    let blocks = args
        .blocks
        .into_iter()
        .enumerate()
        .map(|(at, block)| {
            serde_json::from_value::<CardBlock>(cells_as_text(block))
                .map_err(|err| format!("block {}: {err}", at + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (id, version) = cards::show(
        daemon,
        bot,
        Show {
            card,
            title: args.title,
            blocks,
        },
    )
    .map_err(explain)?;
    Ok(json!({
        "card_id": id,
        "version": version,
        "note": format!(
            "The owner sees the card in your chat. A button pressed or the form sent arrives \
             later as their message that starts with \"Answer on your card {id}\". To change \
             it, call show_card again with card \"{id}\"."
        ),
    }))
}

/// A table's numbers, yes/no and blanks, as the text the card keeps.
fn cells_as_text(mut block: Value) -> Value {
    if block.get("type").and_then(Value::as_str) != Some("table") {
        return block;
    }
    if let Some(rows) = block.get_mut("rows").and_then(Value::as_array_mut) {
        for cell in rows.iter_mut().filter_map(Value::as_array_mut).flatten() {
            let text = match &*cell {
                Value::String(_) => continue,
                Value::Null => String::new(),
                other => other.to_string(),
            };
            *cell = Value::String(text);
        }
    }
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_cells_become_text() {
        let block = cells_as_text(json!({
            "type": "table",
            "columns": ["Item", "Sold", "Ok"],
            "rows": [["Bread", 12, true], ["Cake", 3.5, null]],
        }));
        assert_eq!(
            block["rows"],
            json!([["Bread", "12", "true"], ["Cake", "3.5", ""]])
        );
        let text = json!({ "type": "text", "text": "Hi" });
        assert_eq!(cells_as_text(text.clone()), text);
    }
}
