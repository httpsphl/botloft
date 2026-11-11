//! The programs running now, by file name, for a wait on a program (spec
//! 20.14). A Toolhelp snapshot names every process, even those the daemon
//! may not open.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};

/// The executable names of the running processes, like `ffmpeg.exe`;
/// `None` when Windows would not list them.
pub fn running_programs() -> Option<Vec<String>> {
    // SAFETY: a snapshot of the processes, closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W {
        dwSize: u32::try_from(size_of::<PROCESSENTRY32W>()).unwrap_or(u32::MAX),
        ..Default::default()
    };
    let mut names = Vec::new();
    // SAFETY: `entry` has its size set, as the API needs.
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more {
        let name = &entry.szExeFile;
        let length = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        names.push(String::from_utf16_lossy(&name[..length]));
        // SAFETY: as above, on the same snapshot.
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    // SAFETY: the snapshot was opened above and is closed once.
    let _ = unsafe { CloseHandle(snapshot) };
    Some(names)
}
