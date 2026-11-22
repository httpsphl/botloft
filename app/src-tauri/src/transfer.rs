//! Files in pieces to and from Botloft on another computer (spec 42.7). A
//! download goes where the owner picks in the Save dialog, or into this
//! app's own temporary folder to be opened; an upload reads only the file
//! the owner picked in the Open dialog. So a page can never write or read a
//! file the owner did not choose. The pieces stay base64 from the daemon to
//! here: the page only passes them on.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use tauri::{AppHandle, Manager as _, State};
use tauri_plugin_dialog::DialogExt as _;

/// Most bytes read for the page at once.
const READ_MAX: u64 = 1024 * 1024;

#[derive(Default)]
pub struct Transfers {
    next: AtomicU64,
    downloads: Mutex<HashMap<String, Download>>,
    uploads: Mutex<HashMap<String, PathBuf>>,
}

struct Download {
    file: File,
    path: PathBuf,
    /// In this app's temporary folder, to be opened when done.
    temp: bool,
}

impl Transfers {
    fn id(&self) -> String {
        format!("t{}", self.next.fetch_add(1, Ordering::Relaxed))
    }

    fn start(&self, path: PathBuf, temp: bool) -> Result<String, String> {
        let file = File::create(&path)
            .map_err(|err| format!("could not write {}: {err}", path.display()))?;
        let id = self.id();
        let download = Download { file, path, temp };
        lock(&self.downloads).insert(id.clone(), download);
        Ok(id)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A name without folders or characters Windows refuses, for a file this
/// app writes. Another computer's file names can hold any of them.
fn plain_name(name: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = last
        .chars()
        .map(|ch| {
            if ch.is_control() || r#"<>:"|?*"#.contains(ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_end_matches(['.', ' ']);
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "file".to_owned()
    } else {
        cleaned.to_owned()
    }
}

/// Asks where to save a download called `name`; `None` when the owner
/// cancels.
#[tauri::command]
pub async fn download_save(
    app: AppHandle,
    transfers: State<'_, Transfers>,
    name: String,
) -> Result<Option<String>, String> {
    let name = plain_name(&name);
    let mut dialog = app.dialog().file().set_file_name(&name);
    if let Some(extension) = Path::new(&name).extension().and_then(|ext| ext.to_str()) {
        dialog = dialog.add_filter(extension.to_uppercase(), &[extension]);
    }
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let chosen = tauri::async_runtime::spawn_blocking(move || dialog.blocking_save_file())
        .await
        .map_err(|err| err.to_string())?;
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(|err| err.to_string())?;
    transfers.start(path, false).map(Some)
}

/// A download into this app's own temporary folder, to open when done.
#[tauri::command]
pub async fn download_temp(
    transfers: State<'_, Transfers>,
    name: String,
) -> Result<String, String> {
    let folder = std::env::temp_dir()
        .join("Botloft")
        .join("downloads")
        .join(transfers.id());
    std::fs::create_dir_all(&folder).map_err(|err| err.to_string())?;
    transfers.start(folder.join(plain_name(&name)), true)
}

/// Adds a piece, base64, to a download.
#[tauri::command]
pub async fn download_write(
    transfers: State<'_, Transfers>,
    id: String,
    data: String,
) -> Result<(), String> {
    let bytes = BASE64
        .decode(data.as_bytes())
        .map_err(|err| err.to_string())?;
    let mut downloads = lock(&transfers.downloads);
    let download = downloads.get_mut(&id).ok_or("no such download")?;
    download
        .file
        .write_all(&bytes)
        .map_err(|err| format!("could not write {}: {err}", download.path.display()))
}

/// Ends a download. With `open`, a temporary one opens with the program the
/// system uses for it, when it is a document or media; anything else is
/// refused, as `open_file` does. Without `keep`, the file goes.
#[tauri::command]
pub async fn download_end(
    transfers: State<'_, Transfers>,
    id: String,
    open: bool,
    keep: bool,
) -> Result<(), String> {
    let download = lock(&transfers.downloads)
        .remove(&id)
        .ok_or("no such download")?;
    let Download { file, path, temp } = download;
    drop(file);
    if !keep {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    }
    if open && temp {
        if !crate::open::is_openable(&path) {
            return Err("this kind of file is not opened from Botloft; save it instead".into());
        }
        return crate::open::file(&path);
    }
    Ok(())
}

#[derive(Serialize)]
pub struct Picked {
    id: String,
    name: String,
    size: u64,
}

/// Asks the owner for one file to send, showing `extensions` under `kind`;
/// `None` when they cancel.
#[tauri::command]
pub async fn upload_pick(
    app: AppHandle,
    transfers: State<'_, Transfers>,
    title: String,
    kind: String,
    extensions: Vec<String>,
) -> Result<Option<Picked>, String> {
    let extensions: Vec<&str> = extensions.iter().map(String::as_str).collect();
    let mut dialog = app
        .dialog()
        .file()
        .set_title(&title)
        .add_filter(&kind, &extensions);
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let chosen = tauri::async_runtime::spawn_blocking(move || dialog.blocking_pick_file())
        .await
        .map_err(|err| err.to_string())?;
    let Some(chosen) = chosen else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(|err| err.to_string())?;
    let size = std::fs::metadata(&path)
        .map_err(|err| err.to_string())?
        .len();
    let name = plain_name(&path.file_name().unwrap_or_default().to_string_lossy());
    let id = transfers.id();
    lock(&transfers.uploads).insert(id.clone(), path);
    Ok(Some(Picked { id, name, size }))
}

/// A piece of a picked file, base64: up to `length` bytes from `offset`.
#[tauri::command]
pub async fn upload_read(
    transfers: State<'_, Transfers>,
    id: String,
    offset: u64,
    length: u64,
) -> Result<String, String> {
    let path = lock(&transfers.uploads)
        .get(&id)
        .cloned()
        .ok_or("no such file")?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|err| err.to_string())?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|err| err.to_string())?;
        let mut bytes = Vec::new();
        file.take(length.min(READ_MAX))
            .read_to_end(&mut bytes)
            .map_err(|err| err.to_string())?;
        Ok(BASE64.encode(bytes))
    })
    .await
    .map_err(|err| err.to_string())?
}

#[cfg(test)]
mod tests {
    use super::plain_name;

    #[test]
    fn names_lose_their_folders() {
        assert_eq!(plain_name("report.pdf"), "report.pdf");
        assert_eq!(plain_name("../../secret.txt"), "secret.txt");
        assert_eq!(plain_name(r"C:\Windows\notes.md"), "notes.md");
        assert_eq!(plain_name(".."), "file");
        assert_eq!(plain_name(""), "file");
        assert_eq!(plain_name("a:b?.txt"), "a_b_.txt");
        assert_eq!(plain_name("notes. "), "notes");
    }
}
