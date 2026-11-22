//! The agents run as another user only on a Linux server (spec 42.5): on
//! Windows they always run as the owner.

use std::path::PathBuf;
use std::process::Command;

pub fn use_agent_user(_name: &str) -> Result<(), String> {
    Err("the agents run as another user only on Linux".to_owned())
}

/// Never another user here.
pub fn agents_apart() -> bool {
    false
}

/// The owner's home, where their Claude Code keeps its login.
pub fn agents_home() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn as_agent(_command: &mut Command) {}
