//! The desktop tools (spec 24.6): the bot lists the owner's windows, reads
//! one through its accessibility tree or takes its picture. Each app asks
//! the owner first (spec 24.2), what is never granted stays out (24.3), and
//! the owner must be at their computer (24.8).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::chat::DESKTOP_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopGrant, DesktopLevel};
use serde_json::{Value, json};
use tracing::debug;

use super::calls::explain;
use super::desktop_list::{covering, list};
use crate::approvals::{self, Answer};
use crate::platform::desktop::{self as platform, DesktopError, Window, never, render};
use crate::service::desktop::changed;
use crate::state::Daemon;

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
}

impl Tool {
    fn parse(name: &str, arguments: &Value) -> Result<Self, String> {
        let window = || {
            arguments["window"]
                .as_u64()
                .ok_or_else(|| "Say which window, by its number from desktop_windows.".to_owned())
        };
        let why = || {
            arguments["why"]
                .as_str()
                .map(str::trim)
                .filter(|why| !why.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| "Say why you need it, in one short sentence.".to_owned())
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

fn describe(err: &DesktopError) -> String {
    match err {
        DesktopError::Unavailable => {
            "Using the owner's desktop is not available on this system yet.".to_owned()
        }
        DesktopError::Gone => {
            "That window is not open anymore. Call desktop_windows to see what is open.".to_owned()
        }
        DesktopError::Minimized => "That window is minimized, so there is nothing to see in it. \
            Ask the owner to bring it back if you need it."
            .to_owned(),
        DesktopError::System(why) => format!("Windows could not do it: {why}"),
    }
}

/// Runs a platform call off the async threads: they reach other processes.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, DesktopError> + Send + 'static,
) -> Result<T, String> {
    match tokio::task::spawn_blocking(work).await {
        Ok(result) => result.map_err(|err| describe(&err)),
        Err(_) => Err("Botloft could not reach the desktop.".to_owned()),
    }
}

async fn run(daemon: &Daemon, bot: &BotId, generation: u64, tool: Tool) -> Result<Reply, String> {
    daemon
        .desktop
        .owner_here()
        .map_err(|away| away.why().to_owned())?;
    let windows = blocking(platform::windows).await?;
    let grants = daemon
        .store()
        .desktop_grants(bot)
        .map_err(|err| explain(err.into()))?;
    match tool {
        Tool::Windows => Ok(Reply::Text(list(&windows, &grants))),
        Tool::Look { window, from, why } => {
            let window = granted(daemon, bot, generation, &windows, &grants, window, &why).await?;
            let _turn = daemon.desktop.turn().await;
            let id = window.id;
            let controls = blocking(move || platform::read(id)).await?;
            let (text, next) = render(&controls, from);
            let mut reading = format!("{}\n{text}", heading(&window));
            if let Some(next) = next {
                reading.push_str(&format!(
                    "\nThere is more: call desktop_look with from: {next}."
                ));
            }
            Ok(Reply::Text(reading))
        }
        Tool::Screenshot { window, why } => {
            let window = granted(daemon, bot, generation, &windows, &grants, window, &why).await?;
            let _turn = daemon.desktop.turn().await;
            let id = window.id;
            let picture = blocking(move || platform::picture(id)).await?;
            if picture.jpeg.is_empty() {
                return Err("The picture of that window came out empty.".to_owned());
            }
            Ok(Reply::Picture(
                BASE64.encode(&picture.jpeg),
                heading(&window),
            ))
        }
    }
}

fn heading(window: &Window) -> String {
    format!(
        "Window {}: \"{}\" ({})",
        window.id, window.title, window.app.name
    )
}

/// The window `id`, once the bot may see its app: asks the owner the first
/// time (spec 24.2).
async fn granted(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    windows: &[Window],
    grants: &[DesktopGrant],
    id: u64,
    why: &str,
) -> Result<Window, String> {
    let window = windows
        .iter()
        .find(|window| window.id == id)
        .cloned()
        .ok_or_else(|| describe(&DesktopError::Gone))?;
    if let Some(never) = never(&window) {
        return Err(format!(
            "You may never use that window: {}. Ask the owner if you need something there.",
            never.why()
        ));
    }
    if covering(grants, &window, DesktopLevel::See).is_some() {
        return Ok(window);
    }
    let app = &window.app;
    let path = app.path.to_string_lossy().into_owned();
    let input = json!({
        "app": app.name,
        "path": path,
        "level": DesktopLevel::See.as_str(),
        "why": why,
    });
    match approvals::ask(daemon, bot, generation, DESKTOP_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            daemon
                .store()
                .grant_desktop_app(
                    bot,
                    &path,
                    &app.name,
                    DesktopLevel::See,
                    daemon.clock.now_ms(),
                )
                .map_err(|err| explain(err.into()))?;
            changed(daemon, bot).map_err(explain)?;
            Ok(window)
        }
        Some(Answer::Denied { note }) => {
            let said = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Err(format!(
                "The owner did not let you see {}.{said} Do without it or ask them.",
                app.name
            ))
        }
        Some(Answer::Expired) => Err(format!(
            "The owner did not answer about {} in time. Try again later or do without it.",
            app.name
        )),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}
