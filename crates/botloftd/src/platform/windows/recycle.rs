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
mod tests;
