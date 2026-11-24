//! The system user the agents' browser runs as, apart from the daemon's and
//! the agents' (spec 42.5). A page that broke out of the browser would hold
//! neither the daemon's secrets nor the agents' files, and an agent with a
//! shell cannot read the browsers' cookies, the owner's site logins among
//! them. The profiles live in that user's home, which the daemon reaches
//! through its group.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use super::agent_user::{AgentUser, switch, switchable};

static BROWSER: OnceLock<AgentUser> = OnceLock::new();

/// From now on the browsers run as `name`. Fails, and nothing changes, as
/// the agents' user does (`use_agent_user`).
pub fn use_browser_user(name: &str) -> Result<(), String> {
    BROWSER
        .set(switchable(name)?)
        .map_err(|_| "the browsers' user was already set".to_owned())
}

/// Where the browsers keep their profiles, when they run apart.
pub fn browser_profiles() -> Option<PathBuf> {
    BROWSER.get().map(|user| user.home.join("profiles"))
}

/// Makes `command` run as the browsers' user, when there is one.
pub fn as_browser(command: &mut Command) {
    if let Some(user) = BROWSER.get() {
        switch(command, user);
    }
}

/// Deletes a browser profile. The browser makes its folders open only to
/// itself, so apart they go as its user.
pub fn remove_profile(profile: &Path) -> io::Result<()> {
    let Some(user) = BROWSER.get() else {
        return std::fs::remove_dir_all(profile);
    };
    if !profile.exists() {
        return Err(io::ErrorKind::NotFound.into());
    }
    let mut command = Command::new("rm");
    command.arg("-rf").arg("--").arg(profile).current_dir("/");
    switch(&mut command, user);
    let status = command.status()?;
    // What the browser left is gone; the folder itself is the daemon's.
    let _ = std::fs::remove_dir_all(profile);
    if status.success() && !profile.exists() {
        Ok(())
    } else {
        Err(io::Error::other("the browser's files are still there"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_browser_user_that_is_not_there_is_refused() {
        assert!(use_browser_user("botloft-no-such-browser").is_err());
        assert!(use_browser_user("root").is_err());
        assert_eq!(browser_profiles(), None);
    }
}
