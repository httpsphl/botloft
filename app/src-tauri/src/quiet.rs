//! Whether the screen should stay quiet (spec 31.6): when Windows would not
//! show a notification (a game or a video in full screen, a presentation,
//! "Do not disturb"), the island only shows its pill, and the app's own
//! notices follow Windows' rule.

/// Whether Windows would hold back a notification right now. `false` when
/// it cannot tell, and always outside Windows.
pub fn now() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Shell::{QUNS_ACCEPTS_NOTIFICATIONS, SHQueryUserNotificationState};
        // SAFETY: the call only fills in a state; it takes no pointers of ours.
        match unsafe { SHQueryUserNotificationState() } {
            Ok(state) => state != QUNS_ACCEPTS_NOTIFICATIONS,
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}
