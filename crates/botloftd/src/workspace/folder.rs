//! A work folder the owner picks for a crew (spec 5). Every bot of the crew
//! may edit it like its own folder, so it must not reach Botloft's data or
//! the other crews.

use std::io;
use std::path::{Component, Path, PathBuf};

use crate::paths::Paths;

/// Checks `input` and creates the folder if it is missing. The error is a
/// sentence for the owner.
pub fn choose(paths: &Paths, input: &str) -> Result<PathBuf, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("the work folder must not be empty".to_owned());
    }
    let path = Path::new(input);
    if !path.is_absolute() {
        return Err(r"the work folder must be a full path, like C:\Projects\Site".to_owned());
    }
    let path =
        std::path::absolute(path).map_err(|err| format!("the work folder is not valid: {err}"))?;
    if path.parent().is_none() {
        return Err("pick a folder for the work folder, not a whole drive".to_owned());
    }
    if overlaps(&path, &paths.home) {
        return Err(
            "the work folder cannot be inside Botloft's data folder or contain it".to_owned(),
        );
    }
    if contains(&path, &paths.workspaces_root) {
        return Err("the work folder cannot contain the folders of every bot".to_owned());
    }
    create(&path).map_err(|err| format!("could not create the work folder: {err}"))?;
    Ok(path)
}

fn create(path: &Path) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    if path.is_dir() {
        Ok(())
    } else {
        Err(io::Error::other("a file already has that name"))
    }
}

/// Windows paths ignore case; `\?\` and plain forms of a drive path match.
fn parts(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(prefix) => Some(match prefix.kind() {
                std::path::Prefix::VerbatimDisk(drive) | std::path::Prefix::Disk(drive) => {
                    format!("{}:", char::from(drive).to_ascii_lowercase())
                }
                _ => prefix.as_os_str().to_string_lossy().to_lowercase(),
            }),
            Component::RootDir | Component::CurDir => None,
            Component::ParentDir => Some("..".to_owned()),
            Component::Normal(part) => Some(part.to_string_lossy().to_lowercase()),
        })
        .collect()
}

/// Whether `outer` is `inner` or one of its parents.
fn contains(outer: &Path, inner: &Path) -> bool {
    let (outer, inner) = (parts(outer), parts(inner));
    inner.len() >= outer.len() && inner[..outer.len()] == outer[..]
}

fn overlaps(a: &Path, b: &Path) -> bool {
    contains(a, b) || contains(b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(dir: &Path) -> Paths {
        Paths::new(dir.join("Data").join("Botloft"), dir.join("Botloft"))
    }

    #[test]
    fn a_folder_of_the_owner_is_created_and_kept() {
        let dir = tempfile::tempdir().expect("tempdir");
        let wanted = dir.path().join("Projects").join("Site");
        let chosen = choose(&paths(dir.path()), wanted.to_str().expect("utf-8")).expect("chosen");
        assert_eq!(chosen, wanted);
        assert!(wanted.is_dir());
    }

    #[test]
    fn botloft_data_the_bot_folders_and_drives_are_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = paths(dir.path());
        let refused = [
            String::new(),
            r"relative\folder".to_owned(),
            paths.home.join("secrets").display().to_string(),
            dir.path().join("Data").display().to_string(),
            paths.workspaces_root.display().to_string(),
            dir.path().display().to_string(),
        ];
        for input in refused {
            assert!(choose(&paths, &input).is_err(), "{input:?} was accepted");
        }
        let root = dir.path().ancestors().last().expect("root");
        assert!(choose(&paths, root.to_str().expect("utf-8")).is_err());
    }

    #[test]
    fn a_folder_inside_the_crew_folders_is_fine() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = paths(dir.path());
        let inside = paths.workspaces_root.join("site").join("out");
        assert!(choose(&paths, inside.to_str().expect("utf-8")).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn case_and_verbatim_prefixes_do_not_hide_an_overlap() {
        assert!(contains(
            Path::new(r"C:\Users\Ana"),
            Path::new(r"\\?\c:\users\ana\AppData\Local\Botloft")
        ));
        assert!(!contains(
            Path::new(r"C:\Users\Ana2"),
            Path::new(r"C:\Users\Ana\x")
        ));
    }
}
