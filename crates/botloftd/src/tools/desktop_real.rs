//! The desktop tools that need the owner's real mouse and keyboard (spec
//! 24.7): keys pressed in a window, a click at a point of its picture, and
//! what accessibility cannot click or type, done with them instead. Only
//! where the owner turned that on for the app, never while they use the
//! computer, and stopping the moment they move.

use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopAction, DesktopActionKind, DesktopGrant, DesktopLevel};

use super::desktop_grant::describe;
use super::desktop_list::covering;
use crate::desktop::STOPPED;
use crate::platform::desktop::{self as platform, DesktopError, Window, keys};
use crate::state::Daemon;

/// What the bot reads when the real mouse and keyboard are off there.
pub(super) fn needs_real(app: &str) -> String {
    format!(
        "That needs the owner's real mouse and keyboard, which they have not turned on for {app}. \
         Ask them in the chat; they turn it on in your details, under their desktop."
    )
}

/// Whether the owner turned the real mouse and keyboard on where `window`
/// is.
pub(super) fn real_allowed(grants: &[DesktopGrant], window: &Window) -> bool {
    covering(grants, window, DesktopLevel::Act).is_some_and(|grant| grant.real_input)
}

/// Runs `real` with the owner's hands free, at the desktop's turn: refused
/// while they use the computer or after they just took over, and kept
/// track of when they take over in the middle.
pub(super) async fn with_real_hands<T: Send + 'static>(
    daemon: &Daemon,
    bot: &BotId,
    real: impl FnOnce() -> Result<T, DesktopError> + Send + 'static,
) -> Result<T, String> {
    let _turn = daemon.desktop.turn().await;
    if daemon.desktop.activity.stopped(bot) {
        return Err(STOPPED.to_owned());
    }
    daemon.desktop.hands_free()?;
    match tokio::task::spawn_blocking(real).await {
        Ok(Err(DesktopError::OwnerTookOver)) => {
            daemon.desktop.owner_took_over();
            Err(describe(&DesktopError::OwnerTookOver))
        }
        Ok(result) => result.map_err(|err| describe(&err)),
        Err(_) => Err("Botloft could not reach the desktop.".to_owned()),
    }
}

/// Presses `text` (like "Ctrl+S") in `window` with the real keyboard.
pub(super) async fn press(
    daemon: &Daemon,
    bot: &BotId,
    grants: &[DesktopGrant],
    window: &Window,
    text: &str,
) -> Result<DesktopAction, String> {
    let keys = keys::parse(text).map_err(|err| format!("{err}."))?;
    if !real_allowed(grants, window) {
        return Err(needs_real(&window.app.name));
    }
    let id = window.id;
    with_real_hands(daemon, bot, move || platform::real_press(id, &keys)).await?;
    Ok(DesktopAction {
        kind: DesktopActionKind::Press,
        target: String::new(),
        option: Some(text.trim().to_owned()),
    })
}

/// Clicks a point of the bot's last picture of `window` with the real
/// mouse: `x` and `y` in that picture's pixels.
pub(super) async fn click_at(
    daemon: &Daemon,
    bot: &BotId,
    grants: &[DesktopGrant],
    window: &Window,
    (x, y): (f64, f64),
) -> Result<DesktopAction, String> {
    if !real_allowed(grants, window) {
        return Err(needs_real(&window.app.name));
    }
    let Some((width, height)) = daemon.desktop.picture_size(bot, window.id) else {
        return Err(
            "Take a desktop_screenshot of that window first: x and y are pixels of its picture."
                .to_owned(),
        );
    };
    if !(0.0..=f64::from(width)).contains(&x) || !(0.0..=f64::from(height)).contains(&y) {
        return Err(format!(
            "That point is outside the picture, which is {width} by {height}."
        ));
    }
    let spot = platform::Spot::Window {
        x: x / f64::from(width),
        y: y / f64::from(height),
    };
    let id = window.id;
    with_real_hands(daemon, bot, move || platform::real_click(id, spot)).await?;
    Ok(DesktopAction {
        kind: DesktopActionKind::Click,
        target: String::new(),
        option: None,
    })
}

/// Does with the real mouse and keyboard what accessibility could not, on
/// `control`: a click at its middle, or its text replaced as a person
/// would, by clicking it, selecting it from start to end and typing.
pub(super) async fn instead(
    daemon: &Daemon,
    bot: &BotId,
    window: &Window,
    rect: Option<[i32; 4]>,
    text: Option<String>,
) -> Result<(), String> {
    let Some([left, top, width, height]) = rect else {
        return Err("That control has no place on the screen to click.".to_owned());
    };
    let spot = platform::Spot::Screen {
        x: left + width / 2,
        y: top + height / 2,
    };
    let id = window.id;
    with_real_hands(daemon, bot, move || {
        platform::real_click(id, spot)?;
        if let Some(text) = text {
            // Ctrl+A is not in every classic field; from the start to the
            // end is.
            for keys in ["Ctrl+Home", "Ctrl+Shift+End"] {
                let keys =
                    keys::parse(keys).map_err(|err| DesktopError::System(err.to_string()))?;
                platform::real_press(id, &keys)?;
            }
            platform::real_type(id, &text)?;
        }
        Ok(())
    })
    .await
}

/// Presses Enter in `window`, after typing (`submit`).
pub(super) async fn enter(daemon: &Daemon, bot: &BotId, window: &Window) -> Result<(), String> {
    let id = window.id;
    let enter = keys::parse("Enter").map_err(|err| err.to_string())?;
    with_real_hands(daemon, bot, move || platform::real_press(id, &enter)).await?;
    Ok(())
}
