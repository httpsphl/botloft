//! What is never granted on the desktop, whatever the owner allows (spec
//! 24.3): Botloft itself, windows running as administrator, terminals and
//! command boxes, and password managers. The secure desktop (UAC, the lock
//! screen) has no windows here at all. Password fields are left out where
//! a window is read.

use super::Window;

/// Why a window is never granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Never {
    /// A bot never approves itself.
    Botloft,
    /// It runs with more rights than the daemon.
    Elevated,
    /// Typing there would skip the bot's rules for commands.
    Terminal,
    /// It keeps passwords or the computer's security.
    Passwords,
}

impl Never {
    /// What the bot reads about it.
    pub fn why(self) -> &'static str {
        match self {
            Self::Botloft => "it is Botloft itself",
            Self::Elevated => "it runs as administrator",
            Self::Terminal => {
                "it runs commands, and you have your own tools for that, under your rules"
            }
            Self::Passwords => "it keeps passwords or the computer's security",
        }
    }
}

const BOTLOFT: &[&str] = &[
    "botloft.exe",
    "botloftd.exe",
    "botloft-app.exe",
    "botloft-setup.exe",
];

/// Terminals, command boxes and the tools that change the system under
/// them.
const TERMINALS: &[&str] = &[
    "cmd.exe",
    "powershell.exe",
    "powershell_ise.exe",
    "pwsh.exe",
    "windowsterminal.exe",
    "wt.exe",
    "openconsole.exe",
    "conhost.exe",
    "wsl.exe",
    "wslhost.exe",
    "bash.exe",
    "mintty.exe",
    "alacritty.exe",
    "wezterm-gui.exe",
    "regedit.exe",
    "mmc.exe",
    "taskmgr.exe",
];

const PASSWORDS: &[&str] = &[
    "1password.exe",
    "bitwarden.exe",
    "keepass.exe",
    "keepassxc.exe",
    "lastpass.exe",
    "dashlane.exe",
    "credentialuibroker.exe",
    "sechealthui.exe",
    "securityhealthsystray.exe",
    "consent.exe",
    "logonui.exe",
];

/// The Credential Manager is a page of the Control Panel, an Explorer
/// window: known by its title, in the languages Botloft speaks.
const CREDENTIALS: &[&str] = &[
    "credential manager",
    "gerenciador de credenciais",
    "administrador de credenciales",
];

/// The class of a dialog box: Explorer's are the Run box (Win+R).
const DIALOG: &str = "#32770";

/// Why `window` is never granted, if it is not.
pub fn never(window: &Window) -> Option<Never> {
    let file = window
        .app
        .path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let named = |list: &[&str]| list.contains(&file.as_str());
    if named(BOTLOFT) {
        return Some(Never::Botloft);
    }
    if window.elevated {
        return Some(Never::Elevated);
    }
    if named(TERMINALS) || (file == "explorer.exe" && window.class == DIALOG) {
        return Some(Never::Terminal);
    }
    let title = window.title.to_lowercase();
    if named(PASSWORDS)
        || (file == "explorer.exe" && CREDENTIALS.iter().any(|name| title.contains(name)))
    {
        return Some(Never::Passwords);
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::App;
    use super::*;

    fn window(path: &str, title: &str, class: &str) -> Window {
        Window {
            id: 1,
            title: title.to_owned(),
            class: class.to_owned(),
            app: App {
                path: PathBuf::from(path),
                name: String::new(),
            },
            minimized: false,
            elevated: false,
        }
    }

    #[test]
    fn everyday_apps_may_be_granted() {
        for path in [
            r"C:\Program Files\Microsoft Office\root\Office16\EXCEL.EXE",
            r"C:\Windows\System32\notepad.exe",
            r"C:\Users\ana\AppData\Local\Clinic\clinic.exe",
        ] {
            assert_eq!(never(&window(path, "Book1", "XLMAIN")), None, "{path}");
        }
        // An ordinary Explorer folder window too.
        let folder = window(r"C:\Windows\explorer.exe", "Downloads", "CabinetWClass");
        assert_eq!(never(&folder), None);
    }

    #[test]
    fn botloft_terminals_and_passwords_never_are() {
        let cases = [
            (
                r"C:\Users\ana\AppData\Local\Botloft\Botloft.exe",
                Never::Botloft,
            ),
            (r"C:\Windows\System32\cmd.exe", Never::Terminal),
            (
                r"C:\Program Files\WindowsApps\Microsoft.WindowsTerminal\WindowsTerminal.exe",
                Never::Terminal,
            ),
            (r"C:\Program Files\PowerShell\7\pwsh.exe", Never::Terminal),
            (r"C:\Windows\regedit.exe", Never::Terminal),
            (r"C:\Windows\System32\Taskmgr.exe", Never::Terminal),
            (
                r"C:\Program Files\KeePassXC\KeePassXC.exe",
                Never::Passwords,
            ),
            (
                r"C:\Program Files\Bitwarden\Bitwarden.exe",
                Never::Passwords,
            ),
            (
                r"C:\Windows\SystemApps\Microsoft.SecHealthUI\SecHealthUI.exe",
                Never::Passwords,
            ),
        ];
        for (path, why) in cases {
            assert_eq!(never(&window(path, "Any", "Any")), Some(why), "{path}");
        }
    }

    #[test]
    fn the_run_box_and_the_credential_manager_never_are() {
        let explorer = r"C:\Windows\explorer.exe";
        assert_eq!(
            never(&window(explorer, "Executar", DIALOG)),
            Some(Never::Terminal)
        );
        assert_eq!(
            never(&window(
                explorer,
                "Gerenciador de Credenciais",
                "CabinetWClass"
            )),
            Some(Never::Passwords)
        );
    }

    #[test]
    fn a_window_running_as_administrator_never_is() {
        let mut admin = window(r"C:\Windows\System32\notepad.exe", "Notes", "Notepad");
        admin.elevated = true;
        assert_eq!(never(&admin), Some(Never::Elevated));
    }
}
