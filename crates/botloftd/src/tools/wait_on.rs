//! What `wait_for` waits for (spec 20.14), checked before the wait is set:
//! a file in the bot's folders that is not there yet, a program that runs
//! now, or a page by `http` or `https`.

use std::path::{Component, Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::WaitOn;

use super::calls::explain;
use crate::routines::waits::is_running;
use crate::service::bots;
use crate::state::Daemon;

const PROGRAM_MAX_CHARS: usize = 100;
pub(super) const TEXT_MAX_CHARS: usize = 200;

/// A file the bot waits for: absolute, or relative to the bot's own
/// folder, and inside that folder or the crew's work folder.
pub(super) fn file(daemon: &Daemon, bot: &BotId, path: &str) -> Result<WaitOn, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("path must not be empty".to_owned());
    }
    let (crew, me) = bots::active(&daemon.store(), bot).map_err(explain)?;
    let workspace = daemon.paths.bot_workspace(&crew.slug, &me.slug);
    let work = daemon.paths.work_folder(&crew);
    let asked = workspace.join(path);
    if asked
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("path must not go up with ..".to_owned());
    }
    let asked = std::path::absolute(&asked).map_err(|_| format!("{path} is not a file path"))?;
    let inside = real(&asked).is_some_and(|real| {
        [&work, &workspace]
            .into_iter()
            .filter_map(|root| root.canonicalize().ok())
            .any(|root| real.starts_with(root))
    });
    if !inside {
        return Err(format!(
            "{} is outside your folder and the crew's work folder ({}): wait for a file in one \
             of them",
            asked.display(),
            work.display()
        ));
    }
    if daemon.routines.probe().file_exists(&asked) {
        return Err(format!(
            "{} already exists: there is nothing to wait for",
            asked.display()
        ));
    }
    Ok(WaitOn::File {
        path: asked.display().to_string(),
    })
}

/// The real path of `path`, which may not exist yet: its nearest folder
/// that exists, resolved, with the rest after it.
fn real(path: &Path) -> Option<PathBuf> {
    let mut rest = Vec::new();
    let mut here = path;
    loop {
        if let Ok(found) = here.canonicalize() {
            let mut real = found;
            for part in rest.iter().rev() {
                real.push(part);
            }
            return Some(real);
        }
        rest.push(here.file_name()?.to_owned());
        here = here.parent()?;
    }
}

/// A program that runs now, by its name.
pub(super) fn program(daemon: &Daemon, name: &str) -> Result<WaitOn, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > PROGRAM_MAX_CHARS {
        return Err(format!(
            "program takes a name of 1 to {PROGRAM_MAX_CHARS} characters"
        ));
    }
    if name.contains(['/', '\\']) {
        return Err("program takes the program's name only, like ffmpeg, not a path".to_owned());
    }
    let running = daemon
        .routines
        .probe()
        .programs()
        .ok_or_else(|| "Botloft could not list the programs on this computer".to_owned())?;
    if !is_running(&running, name) {
        return Err(format!(
            "no program called {name} is running now, so there is nothing to wait for; check \
             its name with tasklist on Windows or ps elsewhere"
        ));
    }
    Ok(WaitOn::Program {
        name: name.to_owned(),
    })
}

/// A page by `http` or `https`, and the words it should show.
pub(super) fn page(url: &str, text: &str) -> Result<WaitOn, String> {
    let url = url.trim();
    let parsed = reqwest::Url::parse(url).map_err(|_| format!("{url} is not a web address"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("url must start with http:// or https://".to_owned());
    }
    let text = text.trim();
    if text.is_empty() || text.chars().count() > TEXT_MAX_CHARS {
        return Err(format!(
            "text takes the words the page should show, 1 to {TEXT_MAX_CHARS} characters"
        ));
    }
    Ok(WaitOn::Page {
        url: parsed.to_string(),
        text: text.to_owned(),
    })
}

/// What the bot waits for, as it reads it back.
pub(super) fn describe(on: &WaitOn) -> String {
    match on {
        WaitOn::File { path } => format!("the file {path} to appear"),
        WaitOn::Program { name } => format!("{name} to end"),
        WaitOn::Page { url, text } => format!("the page {url} to show \"{text}\""),
    }
}

#[cfg(test)]
mod tests {
    use super::{page, real};

    #[test]
    fn a_page_needs_a_web_address_and_words() {
        assert!(page("https://shop.test/order/1", "Shipped").is_ok());
        assert!(page("file:///C:/x.html", "Shipped").is_err());
        assert!(page("not a url", "Shipped").is_err());
        assert!(page("https://shop.test", "  ").is_err());
    }

    #[test]
    fn a_path_not_made_yet_resolves_through_its_folder() {
        let dir = tempfile::tempdir().expect("dir");
        let later = dir.path().join("out").join("report.pdf");
        let real = real(&later).expect("real");
        let root = dir.path().canonicalize().expect("root");
        assert!(real.starts_with(&root), "{}", real.display());
        assert!(real.ends_with("out/report.pdf") || real.ends_with(r"out\report.pdf"));
    }
}
