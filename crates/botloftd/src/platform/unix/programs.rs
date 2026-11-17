//! The programs running now, by name, for a wait on a program (spec
//! 20.14), from `ps` on Linux and macOS.

use std::process::{Command, Stdio};

/// The command names of the running processes (macOS gives the whole path;
/// the caller keeps the file name); `None` when `ps` did not answer.
pub fn running_programs() -> Option<Vec<String>> {
    let output = Command::new("ps")
        .args(["-A", "-o", "comm="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}
