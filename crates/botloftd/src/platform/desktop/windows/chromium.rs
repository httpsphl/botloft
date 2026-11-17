//! Windows that show a web page drawn by Chromium: Chrome, Edge and the
//! apps built on Electron (Slack, Discord, VS Code) or WebView2 (Teams).
//! Chromium builds a page's accessibility tree only once a reader asks for
//! it, and a moment later (spec 24.5): the first reading of such a window
//! has only its frame.

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;
use windows::core::BOOL;

use super::super::Control;
use super::{class_of, handle};

/// The class of the window Chromium shows a page in, the one screen
/// readers reach the page through.
pub(super) const PAGE_HOST: &str = "Chrome_RenderWidgetHostHWND";

/// Whether window `id` shows a web page drawn by Chromium.
pub(super) fn shows_page(id: u64) -> bool {
    let mut found = false;
    // SAFETY: the callback only sets `found`, which outlives the call. It
    // stops the walk when it finds one, and Windows then answers "no".
    let _ = unsafe {
        EnumChildWindows(
            Some(handle(id)),
            Some(page_host),
            LPARAM(std::ptr::from_mut(&mut found) as isize),
        )
    };
    found
}

unsafe extern "system" fn page_host(hwnd: HWND, found: LPARAM) -> BOOL {
    if class_of(hwnd) != PAGE_HOST {
        return BOOL(1);
    }
    // SAFETY: `found` is the flag `shows_page` passed in, alive for the call.
    unsafe { *(found.0 as *mut bool) = true };
    BOOL(0)
}

/// Whether a reading has a page's own controls: Chromium puts them under a
/// document.
pub(super) fn has_page(controls: &[Control]) -> bool {
    controls.iter().any(|control| control.kind == "document")
}

#[cfg(test)]
mod tests {
    use super::{Control, has_page};

    #[test]
    fn a_reading_has_the_page_once_it_has_a_document() {
        let control = |kind| Control {
            kind,
            ..Control::default()
        };
        // Only the frame: the title, its buttons, the hidden address bar.
        let frame = [control("text"), control("button"), control("edit")];
        assert!(!has_page(&frame));
        let page = [control("button"), control("document"), control("button")];
        assert!(has_page(&page));
    }
}
