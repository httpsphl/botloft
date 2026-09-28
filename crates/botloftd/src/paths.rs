//! Where the daemon keeps its state and the bots' workspaces (spec 5).

use std::io;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Environment variable that moves the daemon's data directory, so a dev
/// daemon never touches a real installation.
pub const HOME_ENV: &str = "BOTLOFT_HOME";

/// Name of the folder inside a crew that every bot of the crew shares.
pub const SHARED_DIR: &str = "shared";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// Daemon data: database, secrets, logs, config. Absolute.
    pub home: PathBuf,
    /// Parent of every crew folder. Absolute.
    pub workspaces_root: PathBuf,
}

impl Paths {
    pub fn new(home: PathBuf, workspaces_root: PathBuf) -> Self {
        Self {
            home,
            workspaces_root,
        }
    }

    pub fn db(&self) -> PathBuf {
        self.home.join("botloft.db")
    }

    pub fn secrets(&self) -> PathBuf {
        self.home.join("secrets")
    }

    pub fn logs(&self) -> PathBuf {
        self.home.join("logs")
    }

    pub fn default_config(&self) -> PathBuf {
        self.home.join("config.toml")
    }

    /// Held exclusively while a daemon runs for this home (spec 14).
    pub fn lock_file(&self) -> PathBuf {
        self.home.join("botloftd.lock")
    }

    pub fn crew_dir(&self, crew_slug: &str) -> PathBuf {
        self.workspaces_root.join(crew_slug)
    }

    pub fn shared_dir(&self, crew_slug: &str) -> PathBuf {
        self.crew_dir(crew_slug).join(SHARED_DIR)
    }

    pub fn bot_workspace(&self, crew_slug: &str, bot_slug: &str) -> PathBuf {
        self.crew_dir(crew_slug).join(bot_slug)
    }
}

/// `--home` when given (the scheduled task passes it, spec 14), else
/// `BOTLOFT_HOME` when set, otherwise `%LOCALAPPDATA%\Botloft` (never the
/// roaming profile). Relative paths resolve against the current directory.
pub fn resolve_home(flag: Option<&Path>) -> io::Result<PathBuf> {
    let chosen = flag
        .map(|path| path.as_os_str().to_owned())
        .or_else(|| std::env::var_os(HOME_ENV))
        .filter(|value| !value.is_empty());
    match chosen {
        Some(home) => std::path::absolute(home),
        None => default_home()
            .ok_or_else(|| io::Error::other("cannot find the local application data folder")),
    }
}

/// `%LOCALAPPDATA%\Botloft`, the data folder of an installed daemon.
pub fn default_home() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("Botloft"))
}

/// `workspaces_root` from the config, otherwise `%USERPROFILE%\Botloft`.
pub fn resolve_workspaces_root(config: &Config) -> io::Result<PathBuf> {
    match config.workspaces_root() {
        Some(root) => std::path::absolute(root),
        None => dirs::home_dir()
            .map(|dir| dir.join("Botloft"))
            .ok_or_else(|| io::Error::other("cannot find the user profile folder")),
    }
}

/// Converts a path to the form Claude Code matches permission rules against:
/// POSIX separators, lowercase drive as the first segment and the `//` prefix
/// that anchors at the filesystem root (`C:\Users\ana` -> `//c/Users/ana`).
pub fn permission_rule_path(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let drive = char::from(bytes[0]).to_ascii_lowercase();
        return format!("//{drive}{}", &text[2..]);
    }
    format!("//{}", text.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_follows_the_spec() {
        let paths = Paths::new(PathBuf::from("/data"), PathBuf::from("/ws"));
        assert_eq!(paths.db(), Path::new("/data/botloft.db"));
        assert_eq!(paths.secrets(), Path::new("/data/secrets"));
        assert_eq!(paths.shared_dir("docs"), Path::new("/ws/docs/shared"));
        assert_eq!(
            paths.bot_workspace("docs", "writer"),
            Path::new("/ws/docs/writer")
        );
    }

    #[test]
    fn permission_paths_use_posix_form_with_double_slash() {
        let windows = Path::new(r"C:\Users\ana\AppData\Local\Botloft\secrets");
        assert_eq!(
            permission_rule_path(windows),
            "//c/Users/ana/AppData/Local/Botloft/secrets"
        );
        let verbatim = Path::new(r"\\?\D:\data\Botloft");
        assert_eq!(permission_rule_path(verbatim), "//d/data/Botloft");
        assert_eq!(
            permission_rule_path(Path::new("/home/ana/.botloft")),
            "//home/ana/.botloft"
        );
    }
}
