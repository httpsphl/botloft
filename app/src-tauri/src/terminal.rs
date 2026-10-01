//! A terminal window for a command on Linux and macOS (spec 15.2), as the
//! console window on Windows: the owner sees what happens and can type or
//! paste into it. The command writes its exit code to a file that the app
//! waits for, because a terminal may hand the command to a process it
//! already runs and return at once.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long the owner has to finish in the terminal.
const WAIT: Duration = Duration::from_secs(15 * 60);

/// Runs `program` with `args` in a terminal window and waits for it.
/// Returns whether it exited with 0. `clean` names variables the command
/// must not see.
pub fn run(program: &Path, args: &[&str], clean: &[String]) -> Result<bool, String> {
    let dir = private_dir()?;
    let status = dir.join("status");
    let words: Vec<String> = std::iter::once(program.to_string_lossy().into_owned())
        .chain(args.iter().map(|arg| (*arg).to_owned()))
        .map(|word| sh_quote(&word))
        .collect();
    let script = format!(
        "{}; echo $? > {}",
        words.join(" "),
        sh_quote(&status.to_string_lossy())
    );
    let opened = open(&script, clean);
    let result = opened.and_then(|()| wait_for(&status));
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// A folder only the owner can read, for the status file.
fn private_dir() -> Result<PathBuf, String> {
    use std::os::unix::fs::DirBuilderExt as _;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| time.as_nanos());
    let dir = std::env::temp_dir().join(format!("botloft-{}-{nanos}", std::process::id()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .map_err(|err| format!("cannot create {}: {err}", dir.display()))?;
    Ok(dir)
}

fn wait_for(status: &Path) -> Result<bool, String> {
    let deadline = Instant::now() + WAIT;
    while Instant::now() < deadline {
        if let Ok(code) = std::fs::read_to_string(status)
            && !code.trim().is_empty()
        {
            return Ok(code.trim() == "0");
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Ok(false)
}

/// One word for `sh`, in single quotes.
fn sh_quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// Terminal.app runs the script in a new window and comes to the front.
#[cfg(target_os = "macos")]
fn open(script: &str, clean: &[String]) -> Result<(), String> {
    let mut command = Command::new("osascript");
    command.args([
        "-e",
        "tell application \"Terminal\" to activate",
        "-e",
        &format!(
            "tell application \"Terminal\" to do script {}",
            apple_string(script)
        ),
    ]);
    for name in clean {
        command.env_remove(name);
    }
    let status = command
        .status()
        .map_err(|err| format!("cannot open Terminal: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cannot open Terminal ({status})"))
    }
}

/// An AppleScript string literal.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn apple_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The first terminal found on `PATH`, with how it takes a command.
#[cfg(not(target_os = "macos"))]
const TERMINALS: &[(&str, &[&str])] = &[
    ("x-terminal-emulator", &["-e"]),
    ("gnome-terminal", &["--"]),
    ("konsole", &["-e"]),
    ("xfce4-terminal", &["-x"]),
    ("kitty", &[]),
    ("alacritty", &["-e"]),
    ("xterm", &["-e"]),
];

#[cfg(not(target_os = "macos"))]
fn open(script: &str, clean: &[String]) -> Result<(), String> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let found = TERMINALS.iter().find_map(|(name, flags)| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
            .map(|program| (program, *flags))
    });
    let Some((program, flags)) = found else {
        return Err("no terminal was found to sign in with".into());
    };
    let mut command = Command::new(&program);
    command.args(flags).args(["sh", "-c", script]);
    for name in clean {
        command.env_remove(name);
    }
    command
        .spawn()
        .map(drop)
        .map_err(|err| format!("cannot open {}: {err}", program.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_keep_spaces_and_quotes() {
        assert_eq!(
            sh_quote("/Users/Ana Lima/claude"),
            "'/Users/Ana Lima/claude'"
        );
        assert_eq!(sh_quote("it's"), r"'it'\''s'");
        assert_eq!(apple_string(r#"say "hi" \ bye"#), r#""say \"hi\" \\ bye""#);
    }

    #[test]
    fn the_exit_code_comes_from_the_status_file() {
        let dir = private_dir().expect("dir");
        let status = dir.join("status");
        std::fs::write(&status, "0\n").expect("write");
        assert_eq!(wait_for(&status), Ok(true));
        std::fs::write(&status, "1\n").expect("write");
        assert_eq!(wait_for(&status), Ok(false));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
