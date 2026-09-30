//! Moving a folder to the Recycle Bin (spec 7.6), with the shell's
//! `IFileOperation`.
//!
//! When the bin cannot take something (too big for it, a path too long, a
//! drive without a bin) and the shell may not ask, it deletes for good. The
//! sink below refuses exactly that, so the folder stays where it is.

use std::cell::Cell;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::rc::Rc;

use windows::Win32::Foundation::E_ABORT;
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Shell::{
    FOF_ALLOWUNDO, FOF_NO_UI, FOFX_EARLYFAILURE, FOFX_RECYCLEONDELETE, FileOperation,
    IFileOperation, IFileOperationProgressSink, IFileOperationProgressSink_Impl, IShellItem,
    SHCreateItemFromParsingName, TSF_DELETE_RECYCLE_IF_POSSIBLE,
};
use windows::core::{HRESULT, PCWSTR, Ref, Result as ComResult, implement};

/// Moves `path` to the current user's Recycle Bin, or fails and leaves it
/// where it is: nothing is ever deleted for good. `Unsupported` means the
/// bin cannot take it; another error may pass, like a file still in use.
pub fn recycle(path: &Path) -> io::Result<()> {
    let path = path.to_owned();
    // The shell's file operations need a single-threaded apartment, which
    // a thread of the daemon's pool may not be: they get one of their own.
    std::thread::spawn(move || on_this_thread(&path))
        .join()
        .map_err(|_| io::Error::other("the move to the Recycle Bin crashed"))?
}

fn on_this_thread(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "the folder is not there",
        ));
    }
    // SAFETY: plain COM initialization for this thread, which is ours.
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.ok()?;
    let moved = delete_to_bin(path);
    // SAFETY: pairs with the successful CoInitializeEx above; every COM
    // object of `delete_to_bin` is released by now.
    unsafe { CoUninitialize() };
    moved
}

fn delete_to_bin(path: &Path) -> io::Result<()> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let refused = Rc::new(Cell::new(false));
    let sink: IFileOperationProgressSink = Sink {
        refused: Rc::clone(&refused),
    }
    .into();
    // SAFETY: COM is initialized on this thread; `wide` is NUL-terminated
    // and outlives the calls; the interfaces are used on this thread only.
    let (performed, aborted) = unsafe {
        let operation: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;
        operation.SetOperationFlags(
            FOF_NO_UI | FOF_ALLOWUNDO | FOFX_RECYCLEONDELETE | FOFX_EARLYFAILURE,
        )?;
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None)?;
        operation.DeleteItem(&item, &sink)?;
        let performed = operation.PerformOperations();
        let aborted = operation.GetAnyOperationsAborted();
        (performed, aborted)
    };
    if refused.get() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows cannot put this folder in the Recycle Bin",
        ));
    }
    performed?;
    if aborted.is_ok_and(|aborted| aborted.as_bool()) || path.exists() {
        return Err(io::Error::other(
            "Windows did not move the folder to the Recycle Bin",
        ));
    }
    Ok(())
}

/// Stops the operation when an item would be deleted for good.
#[implement(IFileOperationProgressSink)]
struct Sink {
    refused: Rc<Cell<bool>>,
}

#[allow(non_snake_case)]
impl IFileOperationProgressSink_Impl for Sink_Impl {
    fn PreDeleteItem(&self, dwflags: u32, _item: Ref<IShellItem>) -> ComResult<()> {
        // The shell sets this flag on what goes to the bin, and leaves it
        // out on what it is about to destroy.
        if dwflags & TSF_DELETE_RECYCLE_IF_POSSIBLE.0 as u32 == 0 {
            self.refused.set(true);
            return Err(E_ABORT.into());
        }
        Ok(())
    }

    fn StartOperations(&self) -> ComResult<()> {
        Ok(())
    }

    fn FinishOperations(&self, _result: HRESULT) -> ComResult<()> {
        Ok(())
    }

    fn PreRenameItem(&self, _: u32, _: Ref<IShellItem>, _: &PCWSTR) -> ComResult<()> {
        Ok(())
    }

    fn PostRenameItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PreMoveItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PostMoveItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PreCopyItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PostCopyItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PostDeleteItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> ComResult<()> {
        Ok(())
    }

    fn PreNewItem(&self, _: u32, _: Ref<IShellItem>, _: &PCWSTR) -> ComResult<()> {
        Ok(())
    }

    fn PostNewItem(
        &self,
        _: u32,
        _: Ref<IShellItem>,
        _: &PCWSTR,
        _: &PCWSTR,
        _: u32,
        _: HRESULT,
        _: Ref<IShellItem>,
    ) -> ComResult<()> {
        Ok(())
    }

    fn UpdateProgress(&self, _total: u32, _so_far: u32) -> ComResult<()> {
        Ok(())
    }

    fn ResetTimer(&self) -> ComResult<()> {
        Ok(())
    }

    fn PauseTimer(&self) -> ComResult<()> {
        Ok(())
    }

    fn ResumeTimer(&self) -> ComResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::platform::windows::CurrentUser;

    /// Where the bin keeps what came from `original`: its `$I` record and
    /// the `$R` item itself. `None` if the bin has no such thing.
    fn in_bin(original: &Path) -> Option<(PathBuf, PathBuf)> {
        let sid = CurrentUser::query().ok()?.sid_string().ok()?;
        let drive = original.components().next()?.as_os_str().to_string_lossy();
        let bin = PathBuf::from(format!(r"{drive}\$Recycle.Bin\{sid}"));
        let wanted = original.to_string_lossy().to_lowercase();
        for entry in std::fs::read_dir(bin).ok()?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(rest) = name.strip_prefix("$I") else {
                continue;
            };
            // Version, size, time, then the length of the path and the
            // path itself in UTF-16 (the record of Windows 10 and later).
            let Ok(record) = std::fs::read(entry.path()) else {
                continue;
            };
            let units: Vec<u16> = record
                .get(28..)
                .unwrap_or_default()
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes(*pair))
                .take_while(|unit| *unit != 0)
                .collect();
            if String::from_utf16_lossy(&units).to_lowercase() == wanted {
                return Some((
                    entry.path(),
                    entry.path().with_file_name(format!("$R{rest}")),
                ));
            }
        }
        None
    }

    /// Takes the test's own folder out of the bin, so a test run leaves
    /// nothing behind.
    fn purge(record: &Path, item: &Path) {
        let _ = std::fs::remove_dir_all(item);
        let _ = std::fs::remove_file(record);
    }

    #[test]
    fn a_folder_goes_to_the_bin_whole_and_can_be_found_there() {
        let dir = tempfile::tempdir().expect("tempdir");
        let folder = dir.path().join("botloft-recycle-test");
        std::fs::create_dir_all(folder.join("attachments")).expect("folders");
        std::fs::write(folder.join("CLAUDE.md"), "what the bot learned").expect("file");
        std::fs::write(folder.join("attachments").join("plan.txt"), "plan").expect("file");

        recycle(&folder).expect("recycle");

        assert!(!folder.exists(), "the folder left its place");
        let (record, item) = in_bin(&folder).expect("the folder is in the Recycle Bin");
        assert_eq!(
            std::fs::read_to_string(item.join("CLAUDE.md")).expect("kept"),
            "what the bot learned"
        );
        assert!(item.join("attachments").join("plan.txt").is_file());
        purge(&record, &item);
    }

    #[test]
    fn a_folder_with_a_file_in_use_stays_until_the_file_is_let_go() {
        let dir = tempfile::tempdir().expect("tempdir");
        let folder = dir.path().join("botloft-recycle-busy");
        std::fs::create_dir(&folder).expect("folder");
        let log = folder.join("session.log");
        std::fs::write(&log, "still writing").expect("file");
        // As a process that has not ended yet holds its files.
        let held = std::fs::File::open(&log).expect("open");

        let err = recycle(&folder).expect_err("a file is in use");
        assert_ne!(err.kind(), io::ErrorKind::Unsupported, "worth trying again");
        assert_eq!(
            std::fs::read_to_string(&log).expect("kept"),
            "still writing"
        );

        drop(held);
        recycle(&folder).expect("recycle");
        let (record, item) = in_bin(&folder).expect("the folder is in the Recycle Bin");
        purge(&record, &item);
    }

    #[test]
    fn a_missing_folder_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = recycle(&dir.path().join("nothing-here")).expect_err("nothing to move");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn a_folder_with_very_long_paths_is_moved_whole_or_not_at_all() {
        // Paths past 260 characters, as a bot's `node_modules` has. Windows
        // 11 moves the folder whole; whatever a Windows does, nothing of it
        // may be lost.
        let dir = tempfile::tempdir().expect("tempdir");
        let folder = dir.path().join("botloft-recycle-long");
        std::fs::create_dir(&folder).expect("folder");
        // The verbatim form of the path, which may pass 260 characters.
        let top = std::fs::canonicalize(&folder).expect("verbatim path");
        let mut deep = top.clone();
        for _ in 0..8 {
            deep.push("a-folder-name-that-is-forty-characters--");
        }
        std::fs::create_dir_all(&deep).expect("deep folders");
        let file = deep.join("notes.txt");
        std::fs::write(&file, "deep").expect("deep file");

        match recycle(&folder) {
            // Moved whole: it must be in the bin, with the deep file.
            Ok(()) => {
                assert!(!folder.exists());
                let (record, item) = in_bin(&folder).expect("the folder is in the Recycle Bin");
                let mut kept = std::fs::canonicalize(&item).expect("verbatim path");
                kept.extend(deep.strip_prefix(&top).expect("under the folder"));
                let kept = std::fs::read_to_string(kept.join("notes.txt")).expect("deep file");
                assert_eq!(kept, "deep");
                purge(&record, &item);
            }
            // Refused: nothing was touched.
            Err(_) => {
                assert_eq!(std::fs::read_to_string(&file).expect("kept"), "deep");
            }
        }
    }
}
