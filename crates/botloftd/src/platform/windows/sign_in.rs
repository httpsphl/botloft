//! Which Windows sign-in this process runs in, to tell a start after a new
//! sign-in from one in the same sign-in (spec 14).

use windows::Win32::System::RemoteDesktop::{
    WTS_CURRENT_SERVER_HANDLE, WTS_CURRENT_SESSION, WTSFreeMemory, WTSINFOW,
    WTSQuerySessionInformationW, WTSSessionInfo,
};
use windows::core::PWSTR;

/// The session and the time the owner signed in to it. The app and the
/// daemon started by the scheduled task see the same one; signing out and
/// in again, or shutting down, gives a new one. `None` if Windows does not
/// say.
pub fn sign_in_id() -> Option<String> {
    let mut buffer = PWSTR::null();
    let mut bytes = 0u32;
    // SAFETY: on success Windows allocates `buffer`, freed below.
    unsafe {
        WTSQuerySessionInformationW(
            Some(WTS_CURRENT_SERVER_HANDLE),
            WTS_CURRENT_SESSION,
            WTSSessionInfo,
            &mut buffer,
            &mut bytes,
        )
    }
    .ok()?;
    if buffer.is_null() {
        return None;
    }
    let whole = bytes as usize >= size_of::<WTSINFOW>();
    // SAFETY: a WTSSessionInfo query fills one `WTSINFOW`, checked by size.
    let info = whole.then(|| unsafe { buffer.0.cast::<WTSINFOW>().read_unaligned() });
    // SAFETY: `buffer` came from WTSQuerySessionInformationW.
    unsafe { WTSFreeMemory(buffer.0.cast()) };
    let info = info?;
    (info.LogonTime != 0).then(|| format!("{}-{}", info.SessionId, info.LogonTime))
}
