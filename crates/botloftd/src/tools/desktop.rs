//! The desktop tools (spec 24.6): the bot lists the owner's windows, reads
//! one through its accessibility tree or takes its picture, and clicks,
//! types, chooses and scrolls in it. Each app asks the owner first, to see
//! and then to use it (spec 24.2); what is never granted stays out (24.3),
//! and the owner must be at their computer (24.8).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopAction, DesktopLevel};
use serde_json::{Value, json};
use tracing::debug;

use super::calls::explain;
use super::desktop_act::{Ask, act, answered};
use super::desktop_grant::{blocking, granted, heading};
use super::desktop_list::list;
use super::desktop_real::{click_at, press};
use crate::desktop::STOPPED;
use crate::platform::desktop::Window;
use crate::platform::desktop::{self as platform, Action, Scroll, render};
use crate::state::{Daemon, Event};

pub(super) const PREFIX: &str = "desktop_";

enum Tool {
    Windows,
    Look {
        window: u64,
        from: usize,
        why: String,
    },
    Screenshot {
        window: u64,
        why: String,
    },
    Act(Ask),
    /// Keys with the real keyboard (spec 24.7).
    Press {
        window: u64,
        keys: String,
        why: Option<String>,
    },
    /// A click with the real mouse at a point of the window's picture.
    ClickAt {
        window: u64,
        x: f64,
        y: f64,
        why: Option<String>,
    },
}

/// The text argument `name`, trimmed, if it is there.
fn text(arguments: &Value, name: &str) -> Option<String> {
    arguments[name]
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

impl Tool {
    fn parse(name: &str, arguments: &Value) -> Result<Self, String> {
        let window = || {
            arguments["window"]
                .as_u64()
                .ok_or_else(|| "Say which window, by its number from desktop_windows.".to_owned())
        };
        let why = || {
            text(arguments, "why")
                .ok_or_else(|| "Say why you need it, in one short sentence.".to_owned())
        };
        let acting = |action: Action| -> Result<Self, String> {
            let reference = text(arguments, "ref").ok_or_else(|| {
                "Say which control, by its ref from your last reading, like \"d12\".".to_owned()
            })?;
            Ok(Self::Act(Ask {
                reference,
                action,
                why: text(arguments, "why"),
                submit: arguments["submit"].as_bool().unwrap_or(false),
            }))
        };
        Ok(match name.strip_prefix(PREFIX) {
            Some("windows") => Self::Windows,
            Some("look") => Self::Look {
                window: window()?,
                from: arguments["from"].as_u64().unwrap_or(0) as usize,
                why: why()?,
            },
            Some("screenshot") => Self::Screenshot {
                window: window()?,
                why: why()?,
            },
            Some("click") => acting(Action::Click)?,
            // The text as given: spaces may matter in a field.
            Some("type") => acting(Action::Type(
                arguments["text"]
                    .as_str()
                    .ok_or("Say the text to type.")?
                    .to_owned(),
            ))?,
            Some("select") => acting(Action::Select(
                text(arguments, "option").ok_or("Say the option to choose, by its text.")?,
            ))?,
            Some("scroll") => acting(Action::Scroll(match arguments["to"].as_str() {
                Some("up") => Scroll::Up,
                Some("top") => Scroll::Top,
                Some("bottom") => Scroll::Bottom,
                _ => Scroll::Down,
            }))?,
            Some("press") => {
                let keys = text(arguments, "keys").ok_or("Say the keys, like Enter or Ctrl+S.")?;
                // Refused before anything is asked of the owner.
                platform::keys::parse(&keys).map_err(|err| format!("{err}."))?;
                Self::Press {
                    window: window()?,
                    keys,
                    why: text(arguments, "why"),
                }
            }
            Some("click_at") => Self::ClickAt {
                window: window()?,
                x: arguments["x"]
                    .as_f64()
                    .ok_or("Say x, in pixels of the picture.")?,
                y: arguments["y"]
                    .as_f64()
                    .ok_or("Say y, in pixels of the picture.")?,
                why: text(arguments, "why"),
            },
            _ => return Err(format!("There is no tool {name}.")),
        })
    }
}

/// What a desktop tool answers.
enum Reply {
    Text(String),
    /// A JPEG, base64, and a line about the window.
    Picture(String, String),
}

pub(super) async fn call(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    name: &str,
    arguments: Value,
) -> Value {
    debug!(bot = %bot, tool = name, "desktop tool");
    let result = match Tool::parse(name, &arguments) {
        Ok(tool) => run(daemon, bot, generation, tool).await,
        Err(message) => Err(message),
    };
    match result {
        Ok(Reply::Text(text)) => json!({
            "content": [{ "type": "text", "text": text }],
            "isError": false,
        }),
        Ok(Reply::Picture(data, text)) => json!({
            "content": [
                { "type": "image", "data": data, "mimeType": "image/jpeg" },
                { "type": "text", "text": text },
            ],
            "isError": false,
        }),
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true,
        }),
    }
}

async fn run(daemon: &Daemon, bot: &BotId, generation: u64, tool: Tool) -> Result<Reply, String> {
    if daemon.desktop.activity.stopped(bot) {
        return Err(STOPPED.to_owned());
    }
    daemon
        .desktop
        .owner_here()
        .map_err(|away| away.why().to_owned())?;
    let windows = blocking(platform::windows).await?;
    let grants = daemon
        .store()
        .desktop_grants(bot)
        .map_err(|err| explain(err.into()))?;
    let see = DesktopLevel::See;
    match tool {
        Tool::Windows => Ok(Reply::Text(list(&windows, &grants))),
        Tool::Look { window, from, why } => {
            let window = granted(
                daemon,
                bot,
                generation,
                &windows,
                &grants,
                window,
                Some(&why),
                see,
            )
            .await?;
            let _turn = daemon.desktop.turn().await;
            if daemon.desktop.activity.stopped(bot) {
                return Err(STOPPED.to_owned());
            }
            let id = window.id;
            let controls = blocking(move || platform::read(id)).await?;
            let (text, next) = render(&controls, from);
            daemon.desktop.keep(bot, id, controls);
            used(daemon, bot, &window, None);
            let mut reading = format!("{}\n{text}", heading(&window));
            if let Some(next) = next {
                reading.push_str(&format!(
                    "\nThere is more: call desktop_look with from: {next}."
                ));
            }
            Ok(Reply::Text(reading))
        }
        Tool::Screenshot { window, why } => {
            let window = granted(
                daemon,
                bot,
                generation,
                &windows,
                &grants,
                window,
                Some(&why),
                see,
            )
            .await?;
            let _turn = daemon.desktop.turn().await;
            if daemon.desktop.activity.stopped(bot) {
                return Err(STOPPED.to_owned());
            }
            let id = window.id;
            let picture = blocking(move || platform::picture(id)).await?;
            daemon
                .desktop
                .pictured(bot, id, picture.width, picture.height);
            used(daemon, bot, &window, None);
            if picture.jpeg.is_empty() {
                return Err("The picture of that window came out empty.".to_owned());
            }
            Ok(Reply::Picture(
                BASE64.encode(&picture.jpeg),
                heading(&window),
            ))
        }
        Tool::Press { window, keys, why } => {
            let act = DesktopLevel::Act;
            let window = granted(
                daemon,
                bot,
                generation,
                &windows,
                &grants,
                window,
                why.as_deref(),
                act,
            )
            .await?;
            let action = press(daemon, bot, &grants, &window, &keys).await?;
            let said = format!("Pressed {}.", keys.trim());
            answered(daemon, bot, window, said, action)
                .await
                .map(Reply::Text)
        }
        Tool::ClickAt { window, x, y, why } => {
            let act = DesktopLevel::Act;
            let window = granted(
                daemon,
                bot,
                generation,
                &windows,
                &grants,
                window,
                why.as_deref(),
                act,
            )
            .await?;
            let action = click_at(daemon, bot, &grants, &window, (x, y)).await?;
            let said = format!("Clicked at {x}, {y} with the real mouse.");
            answered(daemon, bot, window, said, action)
                .await
                .map(Reply::Text)
        }
        Tool::Act(ask) => act(daemon, bot, generation, &windows, &grants, ask)
            .await
            .map(Reply::Text),
    }
}

/// The bot read or acted in `window`: its panel shows it (spec 24.9).
pub(super) fn used(daemon: &Daemon, bot: &BotId, window: &Window, action: Option<DesktopAction>) {
    let now = daemon.clock.now_ms();
    let state = daemon.desktop.activity.used(bot, window, action, now);
    daemon.emit(Event::DesktopChanged(state));
}
