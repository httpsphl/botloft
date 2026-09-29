//! The owner's name as Windows shows it, for the app's account area.

use windows::Win32::Security::Authentication::Identity::{GetUserNameExW, NameDisplay};
use windows::core::PWSTR;

/// The account's display name ("Ana Lima"), or the user name without one.
pub fn owner_name() -> String {
    display_name()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| std::env::var("USERNAME").unwrap_or_default())
}

fn display_name() -> Option<String> {
    let mut len = 0u32;
    // SAFETY: a size query with no buffer; it fails and sets `len` to the
    // size needed, counting the final NUL.
    let _ = unsafe { GetUserNameExW(NameDisplay, None, &mut len) };
    if len == 0 {
        return None;
    }
    let mut buf = vec![0u16; len as usize];
    // SAFETY: `buf` holds `len` UTF-16 units; on success `len` becomes the
    // length of the name without the NUL.
    let found = unsafe { GetUserNameExW(NameDisplay, Some(PWSTR(buf.as_mut_ptr())), &mut len) };
    found.then(|| String::from_utf16_lossy(&buf[..len as usize]))
}
