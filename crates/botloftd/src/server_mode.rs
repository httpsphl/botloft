//! A daemon on a server (spec 42.2): nobody sits at its screen, so what
//! needs one is refused instead of half working. The rest of the daemon
//! asks [`Daemon::server`](crate::state::Daemon::server).

use botloft_core::protocol::method;

/// What a request needs someone at the screen for, if it does; `None` for
/// everything a server answers as usual. Reads (the grants, the list of
/// away uses) and stops still answer.
pub fn needs_screen(name: &str) -> Option<&'static str> {
    match name {
        method::DESKTOP_APPS
        | method::DESKTOP_GRANT_APP
        | method::DESKTOP_GRANT_WHOLE
        | method::DESKTOP_WATCH
        | method::DESKTOP_RESUME
        | method::DESKTOP_SET_OPTIONS => Some("Desktop Use"),
        method::BROWSER_WINDOW => Some("A browser window to take over"),
        method::LOGINS_OPEN => Some("Signing in to a site in a window"),
        method::CHROME_SET => Some("Your own Chrome"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_what_needs_a_screen_is_refused() {
        for name in [
            method::DESKTOP_GRANT_APP,
            method::DESKTOP_WATCH,
            method::BROWSER_WINDOW,
            method::LOGINS_OPEN,
            method::CHROME_SET,
        ] {
            assert!(needs_screen(name).is_some(), "{name}");
        }
        for name in [
            method::DESKTOP_GRANTS,
            method::DESKTOP_STOP_ALL,
            method::BROWSER_WATCH,
            method::CHROME_STATUS,
            method::MESSAGES_SEND,
            method::SYSTEM_STATUS,
        ] {
            assert_eq!(needs_screen(name), None, "{name}");
        }
    }
}
