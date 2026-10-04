//! What the bot may see of the owner's windows (spec 24.2, 24.4): which
//! grant covers a window, and the list of windows with the titles of apps
//! not granted yet hidden.

use botloft_core::protocol::{DesktopGrant, DesktopLevel, DesktopScope};

use crate::platform::desktop::{Window, never};

fn same_path(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// The grant of `bot` that covers `window` at `level`, if one does.
pub(super) fn covering<'a>(
    grants: &'a [DesktopGrant],
    window: &Window,
    level: DesktopLevel,
) -> Option<&'a DesktopGrant> {
    let path = window.app.path.to_string_lossy();
    grants.iter().find(|grant| {
        grant.level >= level
            && match grant.scope {
                DesktopScope::Desktop => true,
                DesktopScope::App => grant
                    .app_path
                    .as_deref()
                    .is_some_and(|granted| same_path(granted, &path)),
            }
    })
}

/// The windows as the bot may see them (spec 24.4).
pub(super) fn list(windows: &[Window], grants: &[DesktopGrant]) -> String {
    let mut seen = Vec::new();
    let mut others = Vec::new();
    let mut closed: Vec<String> = Vec::new();
    for window in windows {
        if let Some(why) = never(window) {
            let line = format!("{} ({})", window.app.name, why.why());
            if !closed.contains(&line) {
                closed.push(line);
            }
            continue;
        }
        let minimized = if window.minimized { ", minimized" } else { "" };
        if covering(grants, window, DesktopLevel::See).is_some() {
            seen.push(format!(
                "- window {}: \"{}\" ({}{minimized})",
                window.id, window.title, window.app.name
            ));
        } else {
            // The title says what is in it: hidden until the owner allows.
            others.push(format!(
                "- window {}: {}{minimized}",
                window.id, window.app.name
            ));
        }
    }
    let mut text = String::new();
    if seen.is_empty() {
        text.push_str("No open window of an app the owner let you see.");
    } else {
        text.push_str("Windows you may see:\n");
        text.push_str(&seen.join("\n"));
    }
    if !others.is_empty() {
        text.push_str(
            "\n\nOther windows (titles hidden; read one with desktop_look and a why, and the \
             owner is asked to let you see its app):\n",
        );
        text.push_str(&others.join("\n"));
    }
    if !closed.is_empty() {
        text.push_str("\n\nNever available to you: ");
        text.push_str(&closed.join("; "));
    }
    text
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use botloft_core::ids::{BotId, DesktopGrantId};

    use super::*;
    use crate::platform::desktop::App;

    fn window(id: u64, path: &str, title: &str) -> Window {
        Window {
            id,
            title: title.to_owned(),
            class: "Any".to_owned(),
            app: App {
                path: PathBuf::from(path),
                name: path.rsplit('\\').next().unwrap_or(path).to_owned(),
            },
            minimized: false,
            elevated: false,
        }
    }

    fn grant(path: &str, level: DesktopLevel) -> DesktopGrant {
        DesktopGrant {
            id: DesktopGrantId::generate(),
            bot_id: BotId::generate(),
            scope: DesktopScope::App,
            app_path: Some(path.to_owned()),
            app_name: Some("App".to_owned()),
            level,
            real_input: false,
            unattended: false,
            accepted_risks_at: None,
            created_at: 0,
        }
    }

    #[test]
    fn a_grant_covers_its_app_case_aside_and_only_up_to_its_level() {
        let notes = window(1, r"C:\Windows\notepad.exe", "Notes");
        let grants = [grant(r"c:\windows\NOTEPAD.EXE", DesktopLevel::See)];
        assert!(covering(&grants, &notes, DesktopLevel::See).is_some());
        assert!(covering(&grants, &notes, DesktopLevel::Act).is_none());
        let other = window(2, r"C:\Apps\clinic.exe", "Clinic");
        assert!(covering(&grants, &other, DesktopLevel::See).is_none());
    }

    #[test]
    fn the_list_hides_titles_until_granted_and_names_what_is_never_available() {
        let windows = [
            window(1, r"C:\Windows\notepad.exe", "Diary"),
            window(2, r"C:\Apps\clinic.exe", "Patient: Ana Lima"),
            window(3, r"C:\Windows\System32\cmd.exe", "Command Prompt"),
        ];
        let grants = [grant(r"C:\Windows\notepad.exe", DesktopLevel::See)];
        let text = list(&windows, &grants);
        assert!(text.contains("window 1: \"Diary\" (notepad.exe)"), "{text}");
        assert!(text.contains("window 2: clinic.exe"), "{text}");
        assert!(!text.contains("Ana Lima"), "{text}");
        assert!(text.contains("Never available to you: cmd.exe"), "{text}");
        assert!(!text.contains("window 3"), "{text}");
    }
}
