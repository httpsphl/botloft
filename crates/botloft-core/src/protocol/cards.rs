//! Cards a bot shows in its chat (spec 8.11): text, tables, charts, buttons
//! and a form, kept as a chat item the bot can show again, changed. The
//! owner answers one version at a time, with a button or the form, and the
//! answer goes to the bot as the owner's message.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::ChatItemId;

/// Longest title, in characters.
pub const CARD_TITLE_MAX_CHARS: usize = 100;
/// Most blocks in one card.
pub const CARD_BLOCKS_MAX: usize = 20;
/// Longest text block, in characters.
pub const CARD_TEXT_MAX_CHARS: usize = 4_000;
pub const CARD_TABLE_COLUMNS_MAX: usize = 12;
pub const CARD_TABLE_ROWS_MAX: usize = 100;
/// Longest table cell, in characters.
pub const CARD_CELL_MAX_CHARS: usize = 200;
pub const CARD_CHART_LABELS_MAX: usize = 50;
pub const CARD_CHART_SERIES_MAX: usize = 5;
/// Most buttons in one card, across its button blocks.
pub const CARD_BUTTONS_MAX: usize = 6;
pub const CARD_FIELDS_MAX: usize = 10;
/// Fewest and most options of a choice field.
pub const CARD_OPTIONS_MIN: usize = 2;
pub const CARD_OPTIONS_MAX: usize = 10;
/// Longest label of a button, a field, an option, a series or a unit.
pub const CARD_LABEL_MAX_CHARS: usize = 60;
/// Longest value the owner writes in a field, in characters.
pub const CARD_VALUE_MAX_CHARS: usize = 2_000;
/// Largest card, as JSON.
pub const CARD_MAX_BYTES: usize = 32 * 1024;

/// A card in the bot's chat; the chat item's id is the card's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardItem {
    /// 1 when shown, one more each time the bot shows it again.
    pub version: u32,
    pub title: String,
    pub blocks: Vec<CardBlock>,
    /// The owner's answer to this version; `null` until they answer.
    pub answer: Option<CardAnswer>,
}

/// One part of a card, by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum CardBlock {
    /// Markdown.
    Text {
        text: String,
    },
    /// Cells as text, each row as long as `columns`.
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Chart {
        kind: CardChartKind,
        labels: Vec<String>,
        series: Vec<CardSeries>,
        /// What the values count, e.g. "R$" or "%".
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(test, ts(optional))]
        unit: Option<String>,
    },
    Buttons {
        buttons: Vec<CardButton>,
    },
    Form {
        fields: Vec<CardField>,
        /// The send button's label; the app's "Send" when absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(test, ts(optional))]
        submit: Option<String>,
    },
}

text_enum!(
    CardChartKind, "chart kind" {
        Bar => "bar",
        Line => "line",
    }
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardSeries {
    pub name: String,
    /// One per label; `null` where there is no value.
    pub values: Vec<Option<f64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardButton {
    /// Unique in the card: letters, digits, `-` and `_`.
    pub id: String,
    pub label: String,
}

text_enum!(
    CardFieldKind, "field kind" {
        /// One line.
        Text => "text",
        /// Several lines.
        LongText => "long_text",
        Number => "number",
        /// One of `options`.
        Choice => "choice",
        /// Yes or no.
        Check => "check",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardField {
    /// Unique in the card: letters, digits, `-` and `_`.
    pub id: String,
    pub label: String,
    pub kind: CardFieldKind,
    /// For a choice: 2 to 10; empty otherwise.
    #[serde(default)]
    pub options: Vec<String>,
    /// The owner must fill it in; a check never is.
    #[serde(default)]
    pub required: bool,
}

/// What the owner answered to one version of a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardAnswer {
    pub version: u32,
    /// The button pressed; `null` for the form.
    pub button: Option<String>,
    /// What went to the bot, as the owner's message.
    pub text: String,
    /// Unix time in milliseconds.
    pub at: i64,
}

/// `chat.act`: the owner presses a button of a card, or sends its form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatActParams {
    pub item_id: ChatItemId,
    /// The version the owner saw; another one is refused.
    pub version: u32,
    /// The button's id; or `form`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub button: Option<String>,
    /// The form's values by field id: a check is "true" or "false".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub form: Option<BTreeMap<String, String>>,
}
