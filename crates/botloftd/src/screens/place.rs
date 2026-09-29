//! Where a screen lives and how the app reaches it (spec 22.2): one of the
//! bot's two folders, the path inside it, and its `/view` address.

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::ScreenDevice;

use crate::service::{bots, crews};
use crate::state::Daemon;

/// How much of a file is searched for the device it asks for.
const DEVICE_HINT_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Root {
    /// The crew's work folder.
    Work,
    /// The bot's own folder.
    Bot,
}

impl Root {
    fn letter(self) -> &'static str {
        match self {
            Self::Work => "w",
            Self::Bot => "b",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "w" => Some(Self::Work),
            "b" => Some(Self::Bot),
            _ => None,
        }
    }
}

/// A path inside one of the bot's folders, with `/` between parts. Two
/// places are the same file when only the case differs, as on Windows.
#[derive(Debug, Clone)]
pub struct Place {
    pub root: Root,
    pub rel: String,
}

impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.rel.to_lowercase() == other.rel.to_lowercase()
    }
}

impl Eq for Place {}

impl Hash for Place {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root.hash(state);
        self.rel.to_lowercase().hash(state);
    }
}

impl Place {
    /// A place from a `/view` request, refused if any part could leave the
    /// folder or name something hidden.
    pub fn from_request(root: &str, rel: &str) -> Option<Self> {
        let root = Root::parse(root)?;
        let parts: Vec<&str> = rel.split('/').collect();
        let fine = parts.iter().all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && !part.contains(['\\', ':', '\0'])
                && !part.starts_with('~')
        });
        fine.then(|| Self {
            root,
            rel: parts.join("/"),
        })
    }

    /// The file on disk, below `folder`.
    pub fn path_in(&self, folder: &Path) -> PathBuf {
        self.rel
            .split('/')
            .fold(folder.to_owned(), |path, part| path.join(part))
    }

    /// The address the app loads, with `query` (a version) at the end.
    pub fn url(&self, port: u16, key: &str, query: &str) -> String {
        let mut rel = String::new();
        for byte in self.rel.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                    rel.push(char::from(byte));
                }
                _ => rel.push_str(&format!("%{byte:02X}")),
            }
        }
        format!(
            "http://127.0.0.1:{port}/view/{key}/{}/{rel}?{query}",
            self.root.letter()
        )
    }
}

/// The crew's work folder and the bot's folder.
pub fn roots(daemon: &Daemon, bot: &BotId) -> Option<(PathBuf, PathBuf)> {
    let store = daemon.store();
    let record = bots::find(&store, bot).ok()?;
    let crew = crews::find(&store, &record.crew_id).ok()?;
    Some((
        daemon.paths.work_folder(&crew),
        daemon.paths.bot_workspace(&crew.slug, &record.slug),
    ))
}

/// Where an HTML file of the bot is served, if it is in one of its folders.
pub fn locate(daemon: &Daemon, bot: &BotId, path: &Path) -> Option<Place> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
    if extension != "html" && extension != "htm" {
        return None;
    }
    let (work, workspace) = roots(daemon, bot)?;
    [(Root::Work, work), (Root::Bot, workspace)]
        .into_iter()
        .find_map(|(root, folder)| {
            let rel = relative(path, &folder)?;
            Place::from_request(root.letter(), &rel)
        })
}

/// `path` below `folder`, with `/` between parts; ignoring case on Windows.
fn relative(path: &Path, folder: &Path) -> Option<String> {
    let normal = |path: &Path| {
        let text = path.to_string_lossy().replace('/', "\\");
        text.trim_end_matches('\\').to_owned()
    };
    let (path, folder) = (normal(path), normal(folder));
    let starts = if cfg!(windows) {
        path.to_lowercase()
            .starts_with(&format!("{}\\", folder.to_lowercase()))
    } else {
        path.starts_with(&format!("{folder}\\"))
    };
    starts.then(|| {
        path.get(folder.len() + 1..)
            .unwrap_or_default()
            .replace('\\', "/")
    })
}

/// The device an HTML file asks for with `<meta name="botloft-device"
/// content="mobile">` near its start.
pub fn device_of(html: &str) -> Option<ScreenDevice> {
    let end = html.len().min(DEVICE_HINT_BYTES);
    let end = (0..=end).rev().find(|at| html.is_char_boundary(*at))?;
    let head = html[..end].to_ascii_lowercase();
    let at = head.find("botloft-device")?;
    let start = head[..at].rfind('<')?;
    let stop = at + head[at..].find('>')?;
    let tag = &head[start..stop];
    let value = tag.split("content=").nth(1)?;
    let value = value.trim_start_matches(['"', '\'']);
    let value = value.split(['"', '\'', ' ', '/']).next()?;
    ScreenDevice::parse(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_that_could_leave_the_folder_are_refused() {
        assert!(Place::from_request("w", "site/index.html").is_some());
        assert!(Place::from_request("b", "a b/é.html").is_some());
        for bad in [
            "../x.html",
            "a/../../x",
            ".git/config",
            "a//b",
            "",
            "C:/x",
            "a\\b",
            "~$x",
        ] {
            assert!(Place::from_request("w", bad).is_none(), "{bad}");
        }
        assert!(Place::from_request("x", "a.html").is_none());
    }

    #[test]
    fn places_ignore_case_and_make_addresses() {
        let place = Place::from_request("w", "Site/Página inicial.html").expect("place");
        assert_eq!(
            place,
            Place::from_request("w", "site/página INICIAL.html").expect("same")
        );
        assert_eq!(
            place.url(45710, "abc", "rev=2"),
            "http://127.0.0.1:45710/view/abc/w/Site/P%C3%A1gina%20inicial.html?rev=2"
        );
        assert_eq!(
            place.path_in(Path::new(r"C:\Work")),
            Path::new(r"C:\Work")
                .join("Site")
                .join("Página inicial.html")
        );
    }

    #[test]
    fn a_path_is_placed_below_its_folder() {
        assert_eq!(
            relative(Path::new(r"C:\Work\site\a.html"), Path::new(r"C:\Work")).as_deref(),
            Some("site/a.html")
        );
        assert_eq!(
            relative(Path::new(r"C:\Workshop\a.html"), Path::new(r"C:\Work")),
            None
        );
        assert_eq!(
            relative(Path::new("C:/Work/a.html"), Path::new(r"C:\Work\")).as_deref(),
            Some("a.html")
        );
    }

    #[test]
    fn the_device_hint_is_read_from_the_head() {
        let mobile = r#"<html><head><meta name="botloft-device" content="mobile"></head>"#;
        assert_eq!(device_of(mobile), Some(ScreenDevice::Mobile));
        let reversed = "<meta content='Tablet' name='botloft-device'/>";
        assert_eq!(device_of(reversed), Some(ScreenDevice::Tablet));
        assert_eq!(
            device_of("<meta name=\"viewport\" content=\"width=device-width\">"),
            None
        );
        assert_eq!(device_of(""), None);
    }
}
