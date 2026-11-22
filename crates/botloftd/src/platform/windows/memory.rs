//! The machine's memory (spec 42.4): `GlobalMemoryStatusEx`.

use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

use crate::memory::MemoryInfo;

pub fn memory() -> Option<MemoryInfo> {
    let mut status = MEMORYSTATUSEX {
        dwLength: u32::try_from(std::mem::size_of::<MEMORYSTATUSEX>()).ok()?,
        ..Default::default()
    };
    // SAFETY: `status` is a plain struct with its size set, as the call asks.
    unsafe { GlobalMemoryStatusEx(&raw mut status) }.ok()?;
    Some(MemoryInfo {
        total: status.ullTotalPhys,
        available: status.ullAvailPhys,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_machine_has_some_memory_free() {
        let info = memory().expect("memory");
        assert!(info.total > 0);
        assert!(
            info.available > 0 && info.available <= info.total,
            "{info:?}"
        );
    }
}
