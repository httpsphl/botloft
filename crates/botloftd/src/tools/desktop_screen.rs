//! The picture of the owner's whole screen (spec 24.4, D6): only for a bot
//! the owner gave the whole desktop, which is never asked for in the chat
//! (spec 24.2), with the windows never granted covered in black.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::{DesktopGrant, DesktopScope};

use super::desktop_grant::{blocking, used_away};
use crate::desktop::STOPPED;
use crate::platform::desktop as platform;
use crate::state::Daemon;

/// What the owner reads when a bot used the whole screen while away.
const WHOLE: &str = "Desktop";

/// A JPEG of the whole screen, base64, and a line about it.
pub(super) async fn whole(
    daemon: &Daemon,
    bot: &BotId,
    grants: &[DesktopGrant],
) -> Result<(String, String), String> {
    let Some(grant) = grants
        .iter()
        .find(|grant| grant.scope == DesktopScope::Desktop)
    else {
        return Err(
            "The owner has not given you their whole desktop, so pass window: the \
            number of one window from desktop_windows. Only the owner gives the whole desktop, \
            in your details in Botloft."
                .to_owned(),
        );
    };
    if let Err(away) = daemon.desktop.owner_here() {
        if !grant.unattended {
            return Err(away.why().to_owned());
        }
        used_away(daemon, bot, WHOLE);
    }
    let _turn = daemon.desktop.turn().await;
    // The owner may have stopped it while it waited.
    if daemon.desktop.activity.stopped(bot) {
        return Err(STOPPED.to_owned());
    }
    let picture = blocking(platform::screen_picture).await?;
    if picture.jpeg.is_empty() {
        return Err("The picture of the screen came out empty.".to_owned());
    }
    Ok((
        BASE64.encode(&picture.jpeg),
        format!(
            "The owner's whole screen, {} by {}. Windows you may never use are covered in \
             black. To read or use a window, call desktop_windows and pass its number.",
            picture.width, picture.height
        ),
    ))
}
