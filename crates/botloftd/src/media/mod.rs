//! The sounds and videos of a bot that the app plays (spec 8.4):
//! `files.media` hands the app an address on the daemon's own server, and
//! `GET /media/<token>` serves the file in the pieces its player asks for,
//! so a long video plays without crossing the RPC whole.

mod range;
mod serve;

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::BotId;

pub use self::serve::handle as serve;

/// Links kept at once; past this, the oldest stops working.
const LINKS_MAX: usize = 512;

/// The file a link plays: whose it is and where it really is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linked {
    pub bot: BotId,
    pub path: PathBuf,
}

#[derive(Default)]
struct Links {
    by_token: HashMap<String, Linked>,
    /// Tokens, oldest first.
    order: VecDeque<String>,
}

/// The tokens the app plays files with. They live in memory only and change
/// every time the daemon starts, like the screens' keys (spec 22.2).
#[derive(Default)]
pub struct Media {
    links: Mutex<Links>,
}

impl Media {
    fn lock(&self) -> MutexGuard<'_, Links> {
        self.links
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The token for one of a bot's files: the same one while it is kept.
    /// `None` without randomness, since a guessable token is no lock.
    pub fn link(&self, bot: &BotId, path: &Path) -> Option<String> {
        let wanted = Linked {
            bot: bot.clone(),
            path: path.to_path_buf(),
        };
        let mut links = self.lock();
        if let Some((token, _)) = links.by_token.iter().find(|(_, known)| **known == wanted) {
            return Some(token.clone());
        }
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).ok()?;
        let token = hex::encode(bytes);
        links.by_token.insert(token.clone(), wanted);
        links.order.push_back(token.clone());
        while links.order.len() > LINKS_MAX {
            if let Some(oldest) = links.order.pop_front() {
                links.by_token.remove(&oldest);
            }
        }
        Some(token)
    }

    /// The file behind a token, if the token is one of ours.
    pub fn find(&self, token: &str) -> Option<Linked> {
        self.lock().by_token.get(token).cloned()
    }

    /// The bot was deleted: its links stop working.
    pub(crate) fn forget(&self, bot: &BotId) {
        let mut links = self.lock();
        let Links { by_token, order } = &mut *links;
        by_token.retain(|_, linked| &linked.bot != bot);
        order.retain(|token| by_token.contains_key(token));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_keeps_its_token_and_a_deleted_bot_loses_them() {
        let media = Media::default();
        let (scout, writer) = (BotId::generate(), BotId::generate());
        let song = Path::new("C:/work/song.mp3");
        let token = media.link(&scout, song).expect("token");
        assert_eq!(token.len(), 32);
        assert_eq!(media.link(&scout, song).as_deref(), Some(token.as_str()));
        let other = media.link(&writer, song).expect("token");
        assert_ne!(other, token, "each bot has its own link");
        assert_eq!(
            media.find(&token).map(|linked| linked.bot),
            Some(scout.clone())
        );

        media.forget(&scout);
        assert_eq!(media.find(&token), None);
        assert!(media.find(&other).is_some());
        assert_eq!(media.find(""), None);
    }

    #[test]
    fn the_oldest_link_goes_first() {
        let media = Media::default();
        let bot = BotId::generate();
        let first = media.link(&bot, Path::new("0.mp4")).expect("token");
        for index in 1..=LINKS_MAX {
            media.link(&bot, Path::new(&format!("{index}.mp4")));
        }
        assert_eq!(media.find(&first), None);
        assert_eq!(media.lock().by_token.len(), LINKS_MAX);
    }
}
