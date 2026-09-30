//! The Botloft already on this computer, as the NSIS installer left it
//! (spec 15.7): its entry in Installed apps, and the folder it goes to.

use std::path::PathBuf;

/// The app's executable in its folder (`mainBinaryName` in the app's
/// tauri.conf.json).
pub const APP_EXE: &str = "Botloft.exe";

/// What Installed apps says about Botloft.
#[derive(Debug, Default)]
pub struct Installed {
    pub version: Option<String>,
    pub folder: Option<PathBuf>,
}

/// Registry strings may come quoted (`"C:\..."`).
fn unquote(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// Where the NSIS installer puts Botloft, in its own order: the installed
/// folder, the one a past install remembered, or %LOCALAPPDATA%\Botloft.
pub fn target_folder(
    installed: Option<PathBuf>,
    remembered: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
) -> Option<PathBuf> {
    installed
        .or(remembered)
        .or_else(|| local_app_data.map(|base| base.join("Botloft")))
}

#[cfg(windows)]
mod registry {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Botloft";
    /// Tauri's NSIS remembers the install folder under
    /// `Software\<publisher>\<product>`; the publisher is "github", the second
    /// part of the identifier (spec 15.4).
    const FOLDER_KEY: &str = r"Software\github\Botloft";

    pub fn entry(name: &str) -> Option<String> {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(UNINSTALL_KEY)
            .ok()?
            .get_value::<String, _>(name)
            .ok()
    }

    pub fn remembered_folder() -> Option<String> {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(FOLDER_KEY)
            .ok()?
            .get_value::<String, _>("")
            .ok()
    }
}

#[cfg(not(windows))]
mod registry {
    pub fn entry(_name: &str) -> Option<String> {
        None
    }

    pub fn remembered_folder() -> Option<String> {
        None
    }
}

/// Reads Botloft's entry in Installed apps; none when it is not installed.
pub fn read() -> Installed {
    Installed {
        version: registry::entry("DisplayVersion").and_then(|value| unquote(&value)),
        folder: registry::entry("InstallLocation")
            .and_then(|value| unquote(&value))
            .map(PathBuf::from),
    }
}

/// Where this setup's installer will put Botloft.
pub fn folder(installed: &Installed) -> Option<PathBuf> {
    target_folder(
        installed.folder.clone(),
        registry::remembered_folder()
            .and_then(|value| unquote(&value))
            .map(PathBuf::from),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_the_installer_order() {
        let installed = Some(PathBuf::from("C:/A"));
        let remembered = Some(PathBuf::from("C:/B"));
        let base = Some(PathBuf::from("C:/Local"));
        assert_eq!(
            target_folder(installed.clone(), remembered.clone(), base.clone()),
            installed
        );
        assert_eq!(
            target_folder(None, remembered.clone(), base.clone()),
            remembered
        );
        assert_eq!(
            target_folder(None, None, base),
            Some(PathBuf::from("C:/Local").join("Botloft"))
        );
        assert_eq!(target_folder(None, None, None), None);
    }

    #[test]
    fn strips_quotes_and_blanks() {
        assert_eq!(unquote(r#""C:\Botloft""#).as_deref(), Some(r"C:\Botloft"));
        assert_eq!(unquote(" 0.5.0 ").as_deref(), Some("0.5.0"));
        assert_eq!(unquote(r#""""#), None);
    }
}
