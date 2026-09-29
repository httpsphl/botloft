//! Opening Botloft when the owner signs in to Windows (spec 15.2): a value
//! in the current user's Run key, `"<app>" --autostart`. The daemon starts
//! from its own scheduled task (spec 14); this is only the app, near the
//! clock or with its window.

use std::path::Path;

/// Tells the app that Windows opened it at sign-in.
pub const AUTOSTART_ARG: &str = "--autostart";

/// Whether Windows opened this app at sign-in.
pub fn launched_at_sign_in() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_ARG)
}

/// The Run key's command line, quoted: a path with a space, like
/// `C:\Users\Ana Lima\...`, would otherwise be split.
fn command_line(exe: &Path) -> String {
    format!("\"{}\" {AUTOSTART_ARG}", exe.display())
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

/// Opens the app at sign-in, or stops doing so. A dev build never
/// registers itself: it needs the dev server.
pub fn set(on: bool) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    #[cfg(windows)]
    {
        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let command = on.then(|| command_line(&exe));
        registry::set(command.as_deref())
            .map_err(|err| format!("cannot change what opens at sign-in: {err}"))
    }
    #[cfg(not(windows))]
    {
        let _ = on;
        Err("opening at sign-in is only supported on Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::command_line;

    #[test]
    fn the_command_line_is_quoted() {
        assert_eq!(
            command_line(Path::new(
                r"C:\Users\Ana Lima\AppData\Local\Botloft\Botloft.exe"
            )),
            r#""C:\Users\Ana Lima\AppData\Local\Botloft\Botloft.exe" --autostart"#
        );
    }
}
