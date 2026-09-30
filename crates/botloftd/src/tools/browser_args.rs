//! The arguments of the browser tools (spec 21.4), read into one enum.

use serde::Deserialize;
use serde_json::Value;

use super::calls::parse;
use crate::browser::{Key, Scroll, find_key, key_names};

pub(super) const PREFIX: &str = "browser_";

pub(super) enum Tool {
    Open {
        url: String,
    },
    Look {
        from: usize,
    },
    Click {
        reference: String,
    },
    Type {
        reference: String,
        text: String,
        submit: bool,
    },
    Select {
        reference: String,
        option: String,
    },
    Press {
        key: Key,
    },
    Scroll {
        to: Option<Scroll>,
        reference: Option<String>,
    },
    Back,
    Screenshot,
    Close,
    AskOwner {
        task: String,
    },
}

impl Tool {
    /// Whether it does something to the page, beyond reading it or going
    /// to an address.
    pub(super) fn acts(&self) -> bool {
        matches!(
            self,
            Self::Click { .. }
                | Self::Type { .. }
                | Self::Select { .. }
                | Self::Press { .. }
                | Self::Scroll { .. }
                | Self::Back
        )
    }
}

/// Longest request for the owner's help, in characters.
const TASK_MAX: usize = 500;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Open {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Look {
    #[serde(default)]
    from: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Element {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Type {
    #[serde(rename = "ref")]
    reference: String,
    text: String,
    #[serde(default)]
    submit: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Select {
    #[serde(rename = "ref")]
    reference: String,
    option: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Press {
    key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollArgs {
    #[serde(default)]
    to: Option<String>,
    #[serde(default, rename = "ref")]
    reference: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AskOwner {
    task: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Nothing {}

impl Tool {
    /// `name` without Claude Code's `mcp__botloft__` prefix.
    pub(super) fn parse(name: &str, arguments: Value) -> Result<Self, String> {
        let tool = match name.strip_prefix(PREFIX).unwrap_or(name) {
            "open" => parse::<Open>(arguments).map(|args| Self::Open { url: args.url })?,
            "look" => parse::<Look>(arguments).map(|args| Self::Look {
                from: args.from.unwrap_or(0),
            })?,
            "click" => parse::<Element>(arguments).map(|args| Self::Click {
                reference: args.reference,
            })?,
            "type" => parse::<Type>(arguments).map(|args| Self::Type {
                reference: args.reference,
                text: args.text,
                submit: args.submit.unwrap_or(false),
            })?,
            "select" => parse::<Select>(arguments).map(|args| Self::Select {
                reference: args.reference,
                option: args.option,
            })?,
            "press" => {
                let args = parse::<Press>(arguments)?;
                let key = find_key(&args.key).ok_or_else(|| {
                    format!(
                        "{} is not a key browser_press knows: {}",
                        args.key,
                        key_names()
                    )
                })?;
                Self::Press { key }
            }
            "scroll" => {
                let args = parse::<ScrollArgs>(arguments)?;
                let to = match args.to.as_deref().map(str::trim) {
                    None => None,
                    Some("down") => Some(Scroll::Down),
                    Some("up") => Some(Scroll::Up),
                    Some("top") => Some(Scroll::Top),
                    Some("bottom") => Some(Scroll::Bottom),
                    Some(_) => return Err("to must be down, up, top or bottom".to_owned()),
                };
                Self::Scroll {
                    to,
                    reference: args.reference,
                }
            }
            "back" => parse::<Nothing>(arguments).map(|_| Self::Back)?,
            "screenshot" => parse::<Nothing>(arguments).map(|_| Self::Screenshot)?,
            "close" => parse::<Nothing>(arguments).map(|_| Self::Close)?,
            "ask_owner" => {
                let task = parse::<AskOwner>(arguments)?.task.trim().to_owned();
                if task.is_empty() || task.chars().count() > TASK_MAX {
                    return Err(format!(
                        "task must say in one sentence, up to {TASK_MAX} characters, what the \
                         owner should do"
                    ));
                }
                Self::AskOwner { task }
            }
            other => return Err(format!("{PREFIX}{other} is not a browser tool")),
        };
        Ok(tool)
    }
}
