//! The real mouse and keyboard in the tests' own window (spec 24.7). They
//! move the cursor and type on the computer they run on, so they run only
//! with BOTLOFT_REAL_INPUT_TESTS=1, as CI's Windows runner sets.

use super::super::keys::parse;
use super::super::{Control, Spot, read, real_click, real_press, real_type, windows as listed};
use super::test_window::TestWindow;

fn allowed() -> bool {
    let on = std::env::var("BOTLOFT_REAL_INPUT_TESTS").is_ok_and(|value| value == "1");
    if !on {
        eprintln!("real input tests are off: set BOTLOFT_REAL_INPUT_TESTS=1");
    }
    on
}

fn center(control: &Control) -> Spot {
    let [left, top, width, height] = control.rect.expect("a place on the screen");
    Spot::Screen {
        x: left + width / 2,
        y: top + height / 2,
    }
}

fn find(window: &TestWindow, what: impl Fn(&Control) -> bool) -> Control {
    read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| what(control))
        .expect("the control")
}

#[test]
fn a_real_click_and_real_typing_reach_the_window() {
    if !allowed() {
        return;
    }
    let window = TestWindow::open("Botloft desktop test: real");
    let field = find(&window, |control| control.value.as_deref() == Some("hello"));
    real_click(window.id, center(&field)).expect("click the field");
    // Ctrl+A is not in every classic field; from the start to the end is.
    real_press(window.id, &parse("Ctrl+Home").expect("keys")).expect("to the start");
    real_press(window.id, &parse("Ctrl+Shift+End").expect("keys")).expect("select to the end");
    real_type(window.id, "typed for real").expect("type");
    std::thread::sleep(std::time::Duration::from_millis(200));
    let values: Vec<Option<String>> = read(window.id)
        .expect("read")
        .into_iter()
        .map(|control| control.value)
        .collect();
    assert!(
        values.contains(&Some("typed for real".to_owned())),
        "{values:?}"
    );

    let save = find(&window, |control| control.name == "Save");
    real_click(window.id, center(&save)).expect("click Save");
    std::thread::sleep(std::time::Duration::from_millis(300));
    let title = listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .map(|found| found.title);
    assert_eq!(title.as_deref(), Some("Saved"));
}

#[test]
fn every_control_shown_has_its_place_on_the_screen() {
    let window = TestWindow::open("Botloft desktop test: places");
    let save = find(&window, |control| control.name == "Save");
    let [_, _, width, height] = save.rect.expect("a place");
    assert!(width > 0 && height > 0);
}
