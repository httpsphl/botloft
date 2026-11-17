//! The owner's answer on a card, as the message the bot gets (spec 8.11).

use std::collections::BTreeMap;

use super::fail;
use crate::protocol::{CARD_VALUE_MAX_CHARS, CardBlock, CardField, CardFieldKind, CardItem};
use crate::validate::ValidationError;

/// What the owner's answer says, as their message: the button's label, or
/// one "label: value" line per field. Also the button pressed, if one was.
pub fn answer(
    card: &CardItem,
    button: Option<&str>,
    form: Option<&BTreeMap<String, String>>,
) -> Result<(Option<String>, String), ValidationError> {
    match (button, form) {
        (Some(pressed), None) => card
            .blocks
            .iter()
            .flat_map(|block| match block {
                CardBlock::Buttons { buttons } => buttons.as_slice(),
                _ => &[],
            })
            .find(|one| one.id == pressed)
            .map(|one| (Some(one.id.clone()), one.label.clone()))
            .ok_or_else(|| fail(format!("the card has no button \"{pressed}\""))),
        (None, Some(values)) => {
            let fields = card
                .blocks
                .iter()
                .find_map(|block| match block {
                    CardBlock::Form { fields, .. } => Some(fields),
                    _ => None,
                })
                .ok_or_else(|| fail("the card has no form"))?;
            if let Some(unknown) = values
                .keys()
                .find(|key| !fields.iter().any(|f| &f.id == *key))
            {
                return Err(fail(format!("the form has no field \"{unknown}\"")));
            }
            let lines = fields
                .iter()
                .map(|one| line(one, values.get(&one.id).map(String::as_str)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((None, lines.join("\n")))
        }
        _ => Err(fail("press one button or send the form")),
    }
}

/// "Label: value" for one field; "—" for one left empty.
fn line(field: &CardField, value: Option<&str>) -> Result<String, ValidationError> {
    let value = value.map(str::trim).unwrap_or_default();
    let shown = match field.kind {
        CardFieldKind::Check => match value {
            "true" => "✓".to_owned(),
            "" | "false" => "✗".to_owned(),
            _ => return Err(fail(format!("\"{}\" takes true or false", field.label))),
        },
        _ if value.is_empty() => {
            if field.required {
                return Err(fail(format!("\"{}\" must be filled in", field.label)));
            }
            "—".to_owned()
        }
        CardFieldKind::Number => {
            let number: f64 = value
                .replace(',', ".")
                .parse()
                .map_err(|_| fail(format!("\"{}\" takes a number", field.label)))?;
            if !number.is_finite() {
                return Err(fail(format!("\"{}\" takes a number", field.label)));
            }
            value.to_owned()
        }
        CardFieldKind::Choice => {
            if !field.options.iter().any(|option| option == value) {
                return Err(fail(format!(
                    "\"{value}\" is not one of the options of \"{}\"",
                    field.label
                )));
            }
            value.to_owned()
        }
        CardFieldKind::Text | CardFieldKind::LongText => {
            if value.chars().count() > CARD_VALUE_MAX_CHARS {
                return Err(fail(format!(
                    "\"{}\" takes at most {CARD_VALUE_MAX_CHARS} characters",
                    field.label
                )));
            }
            if field.kind == CardFieldKind::Text {
                value.split_whitespace().collect::<Vec<_>>().join(" ")
            } else {
                value.to_owned()
            }
        }
    };
    Ok(format!("{}: {shown}", field.label))
}
