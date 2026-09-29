//! `screens.list` (spec 22.4): the HTML files a bot made that the app can
//! load, newest first, and the ones it is writing that are not on disk yet.

use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

use botloft_core::protocol::{FilesListParams, Screen, ScreenDevice, ScreensListParams};

use super::{ApiResult, files};
use crate::screens::{self, Place, device_of};
use crate::state::Daemon;

/// How much of a file is read for its device hint.
const HEAD_BYTES: u64 = 8 * 1024;

pub fn list(daemon: &Daemon, params: ScreensListParams) -> ApiResult<Vec<Screen>> {
    let bot = params.bot_id;
    let made = files::list(
        daemon,
        FilesListParams {
            bot_id: bot.clone(),
        },
    )?;
    let key = daemon.screens.key(&bot);
    let drafts = daemon.screens.drafts_of(&bot);
    let writing: HashSet<&Place> = drafts.iter().map(|(place, _, _)| place).collect();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for file in made
        .into_iter()
        .filter(|file| file.media_type == "text/html")
    {
        let Some(place) = screens::locate(daemon, &bot, Path::new(&file.path)) else {
            continue;
        };
        out.push(Screen {
            url: place.url(daemon.port, &key, &format!("v={}", file.modified_at)),
            device: device_hint(Path::new(&file.path)),
            writing: writing.contains(&place),
            path: file.path,
            name: file.name,
            folder: file.folder,
            modified_at: file.modified_at,
        });
        seen.insert(place);
    }
    let now = daemon.clock.now_ms();
    for (place, path, content) in drafts.iter().filter(|(place, _, _)| !seen.contains(place)) {
        let name = place.rel.rsplit('/').next().unwrap_or_default().to_owned();
        let folder = place
            .rel
            .rsplit_once('/')
            .map(|(folder, _)| folder.to_owned());
        out.insert(
            0,
            Screen {
                url: place.url(daemon.port, &key, "rev=0"),
                device: device_of(content),
                writing: true,
                path: path.clone(),
                name,
                folder: folder.unwrap_or_default(),
                modified_at: now,
            },
        );
    }
    Ok(out)
}

fn device_hint(path: &Path) -> Option<ScreenDevice> {
    let mut head = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(HEAD_BYTES)
        .read_to_end(&mut head)
        .ok()?;
    device_of(&String::from_utf8_lossy(&head))
}
