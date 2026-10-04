//! The notice on the owner's screen while a bot uses their real mouse and
//! keyboard (spec 24.7): in the language they read the app in, in the
//! bot's color, gone a moment after the bot's last real action. The words
//! live here, in the three languages of the app (spec 15.6): the app is
//! not always open to show them.

use std::time::Duration;

use botloft_core::ids::BotId;

use crate::platform::desktop;
use crate::state::Daemon;

/// How long the notice stays after the last real action.
const LINGER: Duration = Duration::from_secs(3);

/// What the pill says, for `bot`, in `locale`.
pub fn words(locale: &str, bot: &str) -> String {
    if locale.starts_with("pt") {
        format!("{bot} está usando o seu mouse e teclado. Mexa o mouse para parar.")
    } else if locale.starts_with("es") {
        format!("{bot} está usando tu mouse y teclado. Mueve el mouse para detenerlo.")
    } else {
        format!("{bot} is using your mouse and keyboard. Move the mouse to stop it.")
    }
}

/// `#rrggbb` as red, green and blue; Botloft's orange for anything else.
pub fn color(hex: &str) -> (u8, u8, u8) {
    let channel = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
    match (
        hex.strip_prefix('#').map(str::len),
        channel(1),
        channel(3),
        channel(5),
    ) {
        (Some(6), Some(red), Some(green), Some(blue)) => (red, green, blue),
        _ => (0xFF, 0x7A, 0x59),
    }
}

/// Shows the notice for `bot` over window `over`; the number the matching
/// [`after`] waits on.
pub fn before(daemon: &Daemon, bot: &BotId, over: u64) -> u64 {
    let (name, hex) = daemon
        .store()
        .bot(bot)
        .ok()
        .flatten()
        .map(|bot| (bot.name, bot.color))
        .unwrap_or_default();
    desktop::notice_show(&words(&daemon.desktop.locale(), &name), color(&hex), over);
    daemon.desktop.notice_began()
}

/// Takes the notice away a moment later, unless another real action began
/// meanwhile.
pub fn after(daemon: &Daemon, number: u64) {
    let notices = daemon.desktop.notices();
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(async move {
            tokio::time::sleep(LINGER).await;
            if notices.load(std::sync::atomic::Ordering::SeqCst) == number {
                desktop::notice_hide();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notice_speaks_the_owners_language_and_wears_the_bots_color() {
        assert!(words("pt-BR", "Scout").starts_with("Scout está usando o seu mouse"));
        assert!(words("es", "Scout").contains("Mueve el mouse"));
        assert!(words("en", "Scout").contains("Move the mouse to stop it"));
        assert!(words("fr", "Scout").contains("Move the mouse"));
        assert_eq!(color("#5ec8ff"), (0x5E, 0xC8, 0xFF));
        assert_eq!(color("blue"), (0xFF, 0x7A, 0x59));
    }
}
