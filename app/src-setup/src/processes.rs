//! Botloft's own app running from its install folder: whether it is open,
//! and closing it before the installer replaces its files (spec 15.7). The
//! bots are not in it: they run in the daemon's scheduled task (spec 14).

use std::path::Path;

/// Whether `exe` sits directly in `folder`, compared as Windows does:
/// without regard to case or a trailing separator.
pub fn is_in(exe: &Path, folder: &Path) -> bool {
    let normalize = |path: &Path| {
        path.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .replace('/', "\\")
            .to_lowercase()
    };
    exe.parent()
        .is_some_and(|parent| normalize(parent) == normalize(folder))
}

#[cfg(windows)]
mod win {
    use std::path::{Path, PathBuf};

    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
    };
    use windows::core::PWSTR;

    struct Owned(HANDLE);

    impl Drop for Owned {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful call and is closed once.
            let _ = unsafe { CloseHandle(self.0) };
        }
    }

    fn image_path(process: HANDLE) -> Option<PathBuf> {
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        // SAFETY: the buffer and its size match; the handle is open.
        unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            )
        }
        .ok()?;
        Some(PathBuf::from(String::from_utf16_lossy(
            &buffer[..size as usize],
        )))
    }

    /// The processes running `exe_name` from `folder`, open to be closed.
    pub fn find(folder: &Path, exe_name: &str) -> Vec<HandleOf> {
        // SAFETY: a plain snapshot of the process list.
        let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
            return Vec::new();
        };
        let snapshot = Owned(snapshot);
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut found = Vec::new();
        // SAFETY: `entry` has its size set, as the API requires.
        let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) }.is_ok();
        while more {
            let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
            let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if name.eq_ignore_ascii_case(exe_name) {
                let access =
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE;
                // SAFETY: opening a process by id; failure just skips it.
                if let Ok(process) = unsafe { OpenProcess(access, false, entry.th32ProcessID) } {
                    let process = Owned(process);
                    if image_path(process.0).is_some_and(|exe| super::is_in(&exe, folder)) {
                        found.push(HandleOf(process));
                    }
                }
            }
            // SAFETY: same snapshot and entry as above.
            more = unsafe { Process32NextW(snapshot.0, &mut entry) }.is_ok();
        }
        found
    }

    pub struct HandleOf(Owned);

    impl HandleOf {
        /// Ends the process and waits up to 5 s for it to go.
        pub fn close(&self) {
            // SAFETY: the handle was opened with PROCESS_TERMINATE and SYNCHRONIZE.
            unsafe {
                let _ = TerminateProcess(self.0.0, 0);
                let _ = WaitForSingleObject(self.0.0, 5_000);
            }
        }
    }
}

/// Whether the app runs from `folder`.
pub fn running(folder: &Path, exe_name: &str) -> bool {
    #[cfg(windows)]
    return !win::find(folder, exe_name).is_empty();
    #[cfg(not(windows))]
    {
        let _ = (folder, exe_name);
        false
    }
}

/// Closes the app running from `folder`, as the NSIS installer would.
pub fn close(folder: &Path, exe_name: &str) {
    #[cfg(windows)]
    for process in win::find(folder, exe_name) {
        process.close();
    }
    #[cfg(not(windows))]
    let _ = (folder, exe_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Windows paths: `\` separates them only there.
    #[cfg(windows)]
    #[test]
    fn matches_the_folder_as_windows_does() {
        let folder = Path::new(r"C:\Users\Ana Lima\AppData\Local\Botloft");
        assert!(is_in(
            Path::new(r"c:\users\ana lima\appdata\local\botloft\Botloft.exe"),
            folder
        ));
        assert!(is_in(
            Path::new(r"C:\Users\Ana Lima\AppData\Local\Botloft\Botloft.exe"),
            Path::new(r"C:\Users\Ana Lima\AppData\Local\Botloft\")
        ));
        assert!(!is_in(
            Path::new(r"C:\Users\Ana Lima\AppData\Local\Botloft\bin\Botloft.exe"),
            folder
        ));
        assert!(!is_in(Path::new(r"D:\Botloft\Botloft.exe"), folder));
    }
}
