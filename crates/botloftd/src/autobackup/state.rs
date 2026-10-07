//! What the automatic backup remembers between runs, in `autobackup.json`
//! in the data folder (spec 27.10). The passphrase is not here: it is in the
//! credential store.

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use botloft_core::protocol::AutoBackupEvery;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved {
    pub enabled: bool,
    pub every: AutoBackupEvery,
    /// When the last copy this made went up.
    pub last_ok_at: Option<i64>,
    /// What the light copy held when it last went up (or was last seen
    /// unchanged), to tell whether there is something new to send.
    pub fingerprint: Option<String>,
    /// Not before this time; absent means as soon as it can.
    pub next_at: Option<i64>,
    /// The `reason` of the last failure; cleared by the next success.
    pub last_error: Option<String>,
}

/// How long it waits after a copy was sent or found unchanged.
pub fn period(every: AutoBackupEvery) -> Duration {
    match every {
        AutoBackupEvery::Daily => Duration::from_secs(24 * 3600),
        AutoBackupEvery::Weekly => Duration::from_secs(7 * 24 * 3600),
    }
}

/// What is stored; nothing stored, or a file that cannot be read, is off.
pub fn load(file: &Path) -> Saved {
    fs::read(file)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(file: &Path, saved: &Saved) -> io::Result<()> {
    let tmp = file.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec(saved).map_err(io::Error::other)?)?;
    fs::rename(&tmp, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_kept_and_a_broken_file_reads_as_off() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("autobackup.json");
        assert_eq!(load(&file), Saved::default());
        let saved = Saved {
            enabled: true,
            every: AutoBackupEvery::Weekly,
            last_ok_at: Some(5),
            fingerprint: Some("ab".to_owned()),
            next_at: Some(9),
            last_error: Some("offline".to_owned()),
        };
        save(&file, &saved).expect("save");
        assert_eq!(load(&file), saved);
        fs::write(&file, "{not json").expect("write");
        assert!(!load(&file).enabled);
    }

    #[test]
    fn weekly_is_seven_days() {
        assert_eq!(period(AutoBackupEvery::Daily).as_secs(), 86_400);
        assert_eq!(period(AutoBackupEvery::Weekly).as_secs(), 7 * 86_400);
    }
}
