//! Looking for an agent's program (spec 30): the owner's `PATH` as the user
//! has it (not the daemon's), the folders where the usual managers put
//! programs (mise, npm, Volta, Scoop, pnpm, Cargo, Homebrew), and, for a
//! program npm installed as a `.cmd`, the native executable in its package.
//! A `.cmd` cannot be run directly (spec 7.4), the native one can.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How deep the package of an npm program is searched for its executable.
const PACKAGE_DEPTH: usize = 9;

/// The program called `name` (`codex`, `agy`), looked for in `first`, on the
/// owner's `PATH` and in the usual manager folders. `package` is the npm
/// package that has it as a native executable, when there is one.
pub(super) fn program(name: &str, package: Option<&str>, first: &[PathBuf]) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = first.to_vec();
    dirs.extend(path_dirs());
    dirs.extend(manager_dirs());
    for dir in &dirs {
        for file in file_names(name) {
            let candidate = dir.join(file);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        // An npm program is a `.cmd` here and its package has the real one.
        if let Some(package) = package
            && dir.join(format!("{name}.cmd")).is_file()
            && let Some(native) = native_in_package(&dir.join("node_modules").join(package), name)
        {
            return Some(native);
        }
    }
    None
}

#[cfg(windows)]
fn file_names(name: &str) -> Vec<String> {
    vec![format!("{name}.exe")]
}

#[cfg(not(windows))]
fn file_names(name: &str) -> Vec<String> {
    vec![name.to_owned()]
}

/// The `PATH` of the owner's user (the daemon runs as a task and may not see
/// what the owner added later), then the daemon's own.
fn path_dirs() -> Vec<PathBuf> {
    let user = crate::platform::user_environment()
        .ok()
        .and_then(|vars| {
            vars.into_iter()
                .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case("PATH"))
        })
        .map(|(_, value)| value);
    let mut dirs = Vec::new();
    for value in user.into_iter().chain(std::env::var_os("PATH")) {
        dirs.extend(split(&value));
    }
    dirs
}

/// Split on the separator only: a stray quote in a real `PATH` must not
/// swallow the rest (`std::env::split_paths` treats quotes as grouping).
fn split(value: &OsString) -> Vec<PathBuf> {
    let text = value.to_string_lossy();
    let separator = if cfg!(windows) { ';' } else { ':' };
    text.split(separator)
        .map(|part| part.trim().trim_matches('"'))
        .filter(|part| !part.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn manager_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let var = |name: &str| std::env::var_os(name).map(PathBuf::from);
    if let Some(local) = var("LOCALAPPDATA") {
        dirs.push(local.join("mise").join("shims"));
        dirs.push(local.join("pnpm"));
        dirs.push(local.join("Volta").join("bin"));
    }
    if let Some(roaming) = var("APPDATA") {
        dirs.push(roaming.join("npm"));
    }
    if let Some(home) = dirs::home_dir() {
        for relative in [
            ".local/share/mise/shims",
            ".local/bin",
            ".volta/bin",
            ".npm-global/bin",
            ".bun/bin",
            ".cargo/bin",
            "scoop/shims",
        ] {
            dirs.push(
                relative
                    .split('/')
                    .fold(home.clone(), |dir, part| dir.join(part)),
            );
        }
    }
    #[cfg(not(windows))]
    {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }
    dirs
}

/// The native executable called `name` inside an installed npm package.
fn native_in_package(root: &Path, name: &str) -> Option<PathBuf> {
    let wanted = file_names(name).into_iter().next()?;
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().is_some_and(|file| file == wanted.as_str()) && path.is_file() {
                return Some(path);
            }
            if depth < PACKAGE_DEPTH && path.is_dir() {
                stack.push((path, depth + 1));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_program_in_the_first_folders_is_found() {
        let dir = tempfile::tempdir().expect("dir");
        let file = file_names("zed").remove(0);
        std::fs::write(dir.path().join(&file), b"").expect("write");
        let found = program("zed", None, &[dir.path().to_path_buf()]);
        assert_eq!(found, Some(dir.path().join(file)));
        assert_eq!(program("not-a-real-program-x", None, &[]), None);
    }

    #[test]
    fn an_npm_cmd_leads_to_the_native_program_in_its_package() {
        let dir = tempfile::tempdir().expect("dir");
        std::fs::write(dir.path().join("zed.cmd"), b"").expect("cmd");
        let vendor = dir
            .path()
            .join("node_modules")
            .join("@acme")
            .join("zed")
            .join("node_modules")
            .join("@acme")
            .join("zed-win32-x64")
            .join("vendor");
        std::fs::create_dir_all(&vendor).expect("vendor");
        let native = vendor.join(file_names("zed").remove(0));
        std::fs::write(&native, b"").expect("native");
        // On Windows `zed.exe` is not in the folder itself, only the `.cmd`.
        let found = program("zed", Some("@acme/zed"), &[dir.path().to_path_buf()]);
        if cfg!(windows) {
            assert_eq!(found, Some(native));
        } else {
            // Elsewhere there is no `.cmd`: the plain name is what is looked for.
            assert!(found.is_none() || found == Some(native));
        }
    }

    #[test]
    fn the_path_is_split_on_the_separator_only() {
        let separator = if cfg!(windows) { ';' } else { ':' };
        let value = OsString::from(format!("a{separator}\"b c\"{separator}{separator}d"));
        let dirs = split(&value);
        assert_eq!(
            dirs,
            [PathBuf::from("a"), PathBuf::from("b c"), PathBuf::from("d")]
        );
    }
}
