//! With real windows: a window of the test's own, painted red, found in
//! the list, pictured, minimized and closed.

use std::time::Duration;

use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, ShowWindow};

use super::super::{DesktopError, never, read, render, windows as listed};
use super::capture;
use super::test_window::TestWindow;

#[test]
fn the_list_has_the_window_with_its_app() {
    let window = TestWindow::open("Botloft desktop test: list");
    let found = listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .expect("our window is in the list");
    assert_eq!(found.title, "Botloft desktop test: list");
    assert_eq!(found.class, "BotloftDesktopTest");
    let exe = std::env::current_exe().expect("exe");
    assert_eq!(
        found.app.path.to_string_lossy().to_lowercase(),
        exe.to_string_lossy().to_lowercase()
    );
    assert!(!found.app.name.is_empty());
    assert!(!found.minimized);
    assert!(!found.elevated, "the test is not running as administrator");
    assert_eq!(never(&found), None);
}

#[test]
fn a_picture_is_only_the_window_in_its_own_pixels() {
    let window = TestWindow::open("Botloft desktop test: picture");
    let (rgb, width, height, scale) = capture(window.id).expect("picture");
    assert!(scale >= 1.0);
    // 420 × 320 logical pixels, the title bar included, without the
    // invisible border: within a few pixels of it at any display scale.
    let logical = |pixels: u32| f64::from(pixels) / scale;
    assert!((logical(width) - 420.0).abs() < 20.0, "{width} at {scale}");
    assert!(
        (logical(height) - 320.0).abs() < 20.0,
        "{height} at {scale}"
    );
    // The middle of the window is its red background.
    let middle = ((height / 2 * width + width / 2) * 3) as usize;
    assert_eq!(&rgb[middle..middle + 3], [255, 0, 0]);
    let picture = super::super::picture(window.id).expect("jpeg");
    assert_eq!(&picture.jpeg[..2], [0xFF, 0xD8]);
}

#[test]
fn a_minimized_window_is_listed_but_has_nothing_to_see_and_a_closed_one_is_gone() {
    let mut window = TestWindow::open("Botloft desktop test: minimized");
    // SAFETY: a plain request on our own window.
    let _ = unsafe { ShowWindow(super::handle(window.id), SW_MINIMIZE) };
    std::thread::sleep(Duration::from_millis(300));
    let found = listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .expect("still listed");
    assert!(found.minimized);
    assert!(matches!(capture(window.id), Err(DesktopError::Minimized)));
    let id = window.id;
    window.close();
    assert!(matches!(capture(id), Err(DesktopError::Gone)));
    assert!(
        listed()
            .expect("windows")
            .iter()
            .all(|found| found.id != id)
    );
}

#[test]
fn a_window_reads_as_its_controls_and_never_shows_a_password() {
    let window = TestWindow::open("Botloft desktop test: read");
    let controls = read(window.id).expect("read");
    let (text, next) = render(&controls, 0);
    assert_eq!(next, None);
    assert!(text.contains(r#"button "Save""#), "{text}");
    assert!(text.contains(r#"= "hello""#), "{text}");
    assert!(text.contains(r#"check box "Remember" (checked)"#), "{text}");
    assert!(text.contains("the owner types it"), "{text}");
    assert!(!text.contains("secret"), "{text}");
    let field = controls
        .iter()
        .find(|control| control.value.as_deref() == Some("hello"))
        .expect("the field");
    assert!(!field.runtime_id.is_empty());
    // A window that closed has nothing to read.
    let id = window.id;
    drop(window);
    assert!(read(id).is_err());
}
