//! Saving a copy of a bot's file where the owner picks (spec 15.2). The
//! system's Save dialog chooses the place, so a page can never write a file
//! anywhere the owner did not pick.

use std::path::Path;

use tauri::{AppHandle, Manager as _};
use tauri_plugin_dialog::DialogExt as _;

/// Asks where to save a copy of `path` and copies it there. `Ok(false)`
/// when the owner cancels.
pub fn copy_as(app: &AppHandle, path: &Path) -> Result<bool, String> {
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut dialog = app.dialog().file().set_file_name(&name);
    if let Some(extension) = path.extension().and_then(|extension| extension.to_str()) {
        dialog = dialog.add_filter(extension.to_uppercase(), &[extension]);
    }
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.set_parent(&window);
    }
    let Some(chosen) = dialog.blocking_save_file() else {
        return Ok(false);
    };
    let target = chosen.into_path().map_err(|err| err.to_string())?;
    // The system dialog already asked before replacing a file; copying a
    // file onto itself would empty it.
    if same_file(path, &target) {
        return Ok(true);
    }
    std::fs::copy(path, &target)
        .map(drop)
        .map_err(|err| format!("could not save {}: {err}", target.display()))?;
    Ok(true)
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}
