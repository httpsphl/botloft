//! The daemon runs from the scheduled task with no window. Its manifest
//! asks for no console (`consoleAllocationPolicy`, Windows 11 24H2 and
//! later); older Windows still opens one, which the daemon leaves here.

use windows::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

/// Detaches from the console when no other process uses it: Windows made
/// it just for the daemon, so nobody reads it and closing it would kill
/// the daemon. A console shared with a shell (run from a terminal) stays.
pub fn leave_own_console() {
    let mut processes = [0u32; 2];
    // SAFETY: the buffer is valid for its length; the call only fills it.
    let attached = unsafe { GetConsoleProcessList(&mut processes) };
    if attached == 1 {
        // SAFETY: detaching from a console this process alone uses.
        let _ = unsafe { FreeConsole() };
    }
}
