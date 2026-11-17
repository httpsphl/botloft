//! What a card may hold and how an answer reads (spec 8.11). The bot's tool
//! and the owner's `chat.act` both go through here, so the daemon and its
//! tests share one rule.

use std::collections::HashSet;

use crate::protocol::{
    CARD_BLOCKS_MAX, CARD_BUTTONS_MAX, CARD_CELL_MAX_CHARS, CARD_CHART_LABELS_MAX,
    CARD_CHART_SERIES_MAX, CARD_FIELDS_MAX, CARD_LABEL_MAX_CHARS, CARD_MAX_BYTES, CARD_OPTIONS_MAX,
    CARD_OPTIONS_MIN, CARD_TABLE_COLUMNS_MAX, CARD_TABLE_ROWS_MAX, CARD_TEXT_MAX_CHARS,
    CARD_TITLE_MAX_CHARS, CardBlock, CardField, CardFieldKind,
};
use crate::validate::ValidationError;

mod answer;
mod markdown;

pub use answer::answer;
pub use markdown::as_markdown;

fn fail(text: impl Into<String>) -> ValidationError {
    ValidationError(text.into())
}

/// A trimmed one-line label of at most `max` characters.
fn label(what: &str, value: &str, max: usize) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(fail(format!("{what} must not be empty")));
    }
    if value.chars().count() > max || value.chars().any(char::is_control) {
        return Err(fail(format!(
            "{what} must be one line of at most {max} characters"
        )));
    }
    Ok(value.to_owned())
}

/// Ids of buttons and fields: short, plain and unique in the card.
fn id(value: &str, seen: &mut HashSet<String>) -> Result<String, ValidationError> {
    let ok = !value.is_empty()
        && value.len() <= 40
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !ok {
        return Err(fail(format!(
            "the id \"{value}\" must be 1 to 40 letters, digits, - or _"
        )));
    }
    if !seen.insert(value.to_owned()) {
        return Err(fail(format!("the id \"{value}\" is used twice")));
    }
    Ok(value.to_owned())
}

/// Checks and tidies what a bot shows: the title and the blocks.
pub fn check(
    title: &str,
    blocks: Vec<CardBlock>,
) -> Result<(String, Vec<CardBlock>), ValidationError> {
    let title = label("title", title, CARD_TITLE_MAX_CHARS)?;
    if blocks.is_empty() || blocks.len() > CARD_BLOCKS_MAX {
        return Err(fail(format!("a card takes 1 to {CARD_BLOCKS_MAX} blocks")));
    }
    let mut ids = HashSet::new();
    let mut buttons = 0;
    let mut forms = 0;
    let mut kept = Vec::with_capacity(blocks.len());
    for block in blocks {
        kept.push(match block {
            CardBlock::Text { text } => {
                let text = text.trim().to_owned();
                if text.is_empty() || text.chars().count() > CARD_TEXT_MAX_CHARS {
                    return Err(fail(format!(
                        "a text block takes 1 to {CARD_TEXT_MAX_CHARS} characters"
                    )));
                }
                CardBlock::Text { text }
            }
            CardBlock::Table { columns, rows } => table(columns, rows)?,
            CardBlock::Chart {
                kind,
                labels,
                series,
                unit,
            } => {
                if labels.is_empty() || labels.len() > CARD_CHART_LABELS_MAX {
                    return Err(fail(format!(
                        "a chart takes 1 to {CARD_CHART_LABELS_MAX} labels"
                    )));
                }
                if series.is_empty() || series.len() > CARD_CHART_SERIES_MAX {
                    return Err(fail(format!(
                        "a chart takes 1 to {CARD_CHART_SERIES_MAX} series"
                    )));
                }
                let labels: Vec<String> = labels
                    .iter()
                    .map(|one| label("a chart label", one, CARD_LABEL_MAX_CHARS))
                    .collect::<Result<_, _>>()?;
                let mut kept_series = Vec::with_capacity(series.len());
                for mut one in series {
                    one.name = label("a series name", &one.name, CARD_LABEL_MAX_CHARS)?;
                    if one.values.len() != labels.len() {
                        return Err(fail(format!(
                            "the series \"{}\" must have one value per label",
                            one.name
                        )));
                    }
                    if one.values.iter().flatten().any(|value| !value.is_finite()) {
                        return Err(fail("chart values must be numbers"));
                    }
                    kept_series.push(one);
                }
                let unit = match unit.as_deref().map(str::trim) {
                    None | Some("") => None,
                    Some(unit) => Some(label("unit", unit, CARD_LABEL_MAX_CHARS)?),
                };
                CardBlock::Chart {
                    kind,
                    labels,
                    series: kept_series,
                    unit,
                }
            }
            CardBlock::Buttons { buttons: list } => {
                buttons += list.len();
                if list.is_empty() || buttons > CARD_BUTTONS_MAX {
                    return Err(fail(format!(
                        "a card takes 1 to {CARD_BUTTONS_MAX} buttons"
                    )));
                }
                let mut kept_buttons = Vec::with_capacity(list.len());
                for mut button in list {
                    button.id = id(&button.id, &mut ids)?;
                    button.label = label("a button's label", &button.label, CARD_LABEL_MAX_CHARS)?;
                    kept_buttons.push(button);
                }
                CardBlock::Buttons {
                    buttons: kept_buttons,
                }
            }
            CardBlock::Form { fields, submit } => {
                forms += 1;
                if forms > 1 {
                    return Err(fail("a card takes one form at most"));
                }
                if fields.is_empty() || fields.len() > CARD_FIELDS_MAX {
                    return Err(fail(format!("a form takes 1 to {CARD_FIELDS_MAX} fields")));
                }
                let fields = fields
                    .into_iter()
                    .map(|one| field(one, &mut ids))
                    .collect::<Result<_, _>>()?;
                let submit = match submit.as_deref().map(str::trim) {
                    None | Some("") => None,
                    Some(text) => Some(label("submit", text, CARD_LABEL_MAX_CHARS)?),
                };
                CardBlock::Form { fields, submit }
            }
        });
    }
    let size = serde_json::to_string(&kept).map_or(usize::MAX, |json| json.len());
    if size > CARD_MAX_BYTES {
        return Err(fail(format!(
            "the card is too big: keep it under {} KB",
            CARD_MAX_BYTES / 1024
        )));
    }
    Ok((title, kept))
}

fn table(columns: Vec<String>, rows: Vec<Vec<String>>) -> Result<CardBlock, ValidationError> {
    if columns.is_empty() || columns.len() > CARD_TABLE_COLUMNS_MAX {
        return Err(fail(format!(
            "a table takes 1 to {CARD_TABLE_COLUMNS_MAX} columns"
        )));
    }
    if rows.len() > CARD_TABLE_ROWS_MAX {
        return Err(fail(format!(
            "a table takes at most {CARD_TABLE_ROWS_MAX} rows"
        )));
    }
    let columns = columns
        .iter()
        .map(|one| label("a column", one, CARD_LABEL_MAX_CHARS))
        .collect::<Result<Vec<_>, _>>()?;
    for row in &rows {
        if row.len() != columns.len() {
            return Err(fail("each row must have one cell per column"));
        }
        if row
            .iter()
            .any(|cell| cell.chars().count() > CARD_CELL_MAX_CHARS)
        {
            return Err(fail(format!(
                "a cell takes at most {CARD_CELL_MAX_CHARS} characters"
            )));
        }
    }
    Ok(CardBlock::Table { columns, rows })
}

fn field(mut field: CardField, ids: &mut HashSet<String>) -> Result<CardField, ValidationError> {
    field.id = id(&field.id, ids)?;
    field.label = label("a field's label", &field.label, CARD_LABEL_MAX_CHARS)?;
    if field.kind == CardFieldKind::Choice {
        let count = field.options.len();
        if !(CARD_OPTIONS_MIN..=CARD_OPTIONS_MAX).contains(&count) {
            return Err(fail(format!(
                "the choice \"{}\" takes {CARD_OPTIONS_MIN} to {CARD_OPTIONS_MAX} options",
                field.label
            )));
        }
        let mut seen = HashSet::new();
        let mut options = Vec::with_capacity(count);
        for option in &field.options {
            let option = label("an option", option, CARD_LABEL_MAX_CHARS)?;
            if !seen.insert(option.to_lowercase()) {
                return Err(fail(format!("the option \"{option}\" is repeated")));
            }
            options.push(option);
        }
        field.options = options;
    } else if !field.options.is_empty() {
        return Err(fail(format!(
            "only a choice takes options, not \"{}\"",
            field.label
        )));
    }
    if field.kind == CardFieldKind::Check {
        field.required = false;
    }
    Ok(field)
}

#[cfg(test)]
mod tests;
