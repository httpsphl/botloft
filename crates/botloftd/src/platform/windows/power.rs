//! A power request that keeps Windows from sleeping while bots work
//! (spec 14). Unlike `SetThreadExecutionState`, it belongs to a handle and
//! not to a thread, so any Tokio worker can set or clear it, and
//! `powercfg /requests` lists it with the reason below.

use std::io;

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Power::{
    PowerClearRequest, PowerCreateRequest, PowerRequestSystemRequired, PowerSetRequest,
};
use windows::Win32::System::Threading::{
    POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0,
};
use windows::core::PWSTR;

/// `POWER_REQUEST_CONTEXT_VERSION`.
const CONTEXT_VERSION: u32 = 0;
const REASON: &str = "Botloft bots are working";

#[derive(Debug)]
pub struct KeepAwake {
    handle: HANDLE,
    on: bool,
}

// SAFETY: a power request handle can be used and closed from any thread.
unsafe impl Send for KeepAwake {}

impl KeepAwake {
    pub fn new() -> io::Result<Self> {
        let mut reason: Vec<u16> = REASON.encode_utf16().chain(Some(0)).collect();
        let context = REASON_CONTEXT {
            Version: CONTEXT_VERSION,
            Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
            Reason: REASON_CONTEXT_0 {
                SimpleReasonString: PWSTR(reason.as_mut_ptr()),
            },
        };
        // SAFETY: `context` and the string it points at outlive the call,
        // and the request keeps its own copy of the reason.
        let handle = unsafe { PowerCreateRequest(&context) }?;
        Ok(Self { handle, on: false })
    }

    /// Holds the system awake, or lets it sleep again.
    pub fn set(&mut self, on: bool) -> io::Result<()> {
        if on == self.on {
            return Ok(());
        }
        // SAFETY: `handle` is a live power request owned by `self`.
        unsafe {
            if on {
                PowerSetRequest(self.handle, PowerRequestSystemRequired)
            } else {
                PowerClearRequest(self.handle, PowerRequestSystemRequired)
            }
        }?;
        self.on = on;
        Ok(())
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        let _ = self.set(false);
        // SAFETY: closed exactly once.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}
