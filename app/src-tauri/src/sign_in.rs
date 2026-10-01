//! Opening Botloft when the owner signs in (spec 15.2), as
//! `"<app>" --autostart`: a value in the current user's Run key on Windows,
//! an autostart entry on Linux (`~/.config/autostart`), a launch agent on
//! macOS. The daemon starts on its own (spec 14, 14.1); this is only the
//! app, near the clock or with its window.

use std::path::Path;

/// Tells the app that the system opened it at sign-in.
pub const AUTOSTART_ARG: &str = "--autostart";

/// Whether the system opened this app at sign-in.
pub fn launched_at_sign_in() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

/// The Run key's command line, quoted: a path with a space, like
/// `C:\Users\Ana Lima\...`, would otherwise be split.
#[cfg_attr(not(windows), allow(dead_code))]
fn command_line(exe: &Path) -> String {
    format!("\"{}\" {AUTOSTART_ARG}", exe.display())
}

/// The Linux autostart entry.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Botloft\n\
         Comment=Opens Botloft when you sign in\n\
         Exec={} {AUTOSTART_ARG}\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exec_word(&exe.to_string_lossy())
    )
}

/// One word of a desktop entry's `Exec`: quoted, with `"`, `` ` ``, `$` and
/// `\` escaped, then every `\` doubled as a string value wants, and `%`
/// doubled so it is not a field code.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
fn exec_word(word: &str) -> String {
    let mut quoted = String::new();
    for c in word.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    format!("\"{}\"", quoted.replace('\\', "\\\\").replace('%', "%%"))
}

/// The macOS launch agent that opens the app in the owner's session.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn agent_plist(exe: &Path) -> String {
    let exe = exe
        .to_string_lossy()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>io.github.httpsphl.botloft.app</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>{AUTOSTART_ARG}</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>LimitLoadToSessionType</key><string>Aqua</string>
  <key>ProcessType</key><string>Interactive</string>
</dict>
</plist>
"#
    )
}

#[cfg(windows)]
mod registry {
    use std::io;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_BINARY};
    use winreg::types::ToRegValue as _;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    /// Where Task Manager keeps its "Startup apps" switch.
    const APPROVED_KEY: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const VALUE: &str = "Botloft";
    /// What Task Manager writes for an app allowed to start.
    const APPROVED: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    pub fn set(command: Option<&str>) -> io::Result<()> {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let (run, _) = user.create_subkey(RUN_KEY)?;
        let Some(command) = command else {
            return match run.delete_value(VALUE) {
                Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
                _ => Ok(()),
            };
        };
        run.set_raw_value(VALUE, &command.to_reg_value())?;
        // Turned off in Task Manager earlier: turning it on here wins.
        if let Ok(approved) = user.open_subkey_with_flags(APPROVED_KEY, KEY_READ | KEY_SET_VALUE)
            && approved.get_raw_value(VALUE).is_ok()
        {
            approved.set_raw_value(
                VALUE,
                &winreg::RegValue {
                    bytes: APPROVED.to_vec(),
                    vtype: REG_BINARY,
                },
            )?;
        }
        Ok(())
    }
}

/// Writes the file that opens the app at sign-in, or deletes it.
#[cfg(unix)]
fn write_or_remove(path: &Path, contents: Option<String>) -> std::io::Result<()> {
    match contents {
        Some(contents) => {
            if let Some(folder) = path.parent() {
                std::fs::create_dir_all(folder)?;
            }
            std::fs::write(path, contents)
        }
        None => match std::fs::remove_file(path) {
            Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err),
            _ => Ok(()),
        },
    }
}

/// Opens the app at sign-in, or stops doing so. A dev build never
/// registers itself: it needs the dev server.
pub fn set(on: bool) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let failed = |err: std::io::Error| format!("cannot change what opens at sign-in: {err}");
    #[cfg(windows)]
    {
        let command = on.then(|| command_line(&exe));
        registry::set(command.as_deref()).map_err(failed)
    }
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("cannot find the home folder")?;
        let path = home
            .join("Library")
            .join("LaunchAgents")
            .join("io.github.httpsphl.botloft.app.plist");
        write_or_remove(&path, on.then(|| agent_plist(&exe))).map_err(failed)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let config = dirs::config_dir().ok_or("cannot find the configuration folder")?;
        let path = config.join("autostart").join("botloft.desktop");
        write_or_remove(&path, on.then(|| desktop_entry(&exe))).map_err(failed)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn the_command_line_is_quoted() {
        assert_eq!(
            command_line(Path::new(
                r"C:\Users\Ana Lima\AppData\Local\Botloft\Botloft.exe"
            )),
            r#""C:\Users\Ana Lima\AppData\Local\Botloft\Botloft.exe" --autostart"#
        );
    }

    #[test]
    fn the_linux_entry_quotes_the_app() {
        let entry = desktop_entry(Path::new("/opt/Botloft 100%/botloft"));
        assert!(entry.contains("Exec=\"/opt/Botloft 100%%/botloft\" --autostart\n"));
        assert_eq!(exec_word("a\"b"), r#""a\\"b""#);
        assert_eq!(exec_word(r"a\b"), r#""a\\\\b""#);
    }

    #[test]
    fn the_macos_agent_opens_the_app_in_the_session() {
        let plist = agent_plist(Path::new("/Applications/A & B.app/Contents/MacOS/botloft"));
        assert!(
            plist.contains("<string>/Applications/A &amp; B.app/Contents/MacOS/botloft</string>")
        );
        assert!(plist.contains("<string>--autostart</string>"));
        assert!(plist.contains("<key>RunAtLoad</key><true/>"));
    }
}
