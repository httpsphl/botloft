//! Acting through accessibility on real controls (spec 24.5), in the
//! tests' own window: a click, typing, a check box, a combo box, a long
//! list scrolled, and a password field left alone.

use super::super::{Acted, Action, Control, DesktopError, Scroll, act, read, windows as listed};
use super::test_window::{ITEMS, TestWindow};

/// The control whose line has `kind` and `name`, as a reading has it.
fn named(window: &TestWindow, kind: &str, name: &str) -> Control {
    read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.kind == kind && control.name == name)
        .unwrap_or_else(|| panic!("no {kind} {name:?} in the window"))
}

fn with_value(window: &TestWindow, value: &str) -> Control {
    read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.value.as_deref() == Some(value))
        .unwrap_or_else(|| panic!("no control with the value {value:?}"))
}

fn title(window: &TestWindow) -> String {
    listed()
        .expect("windows")
        .into_iter()
        .find(|found| found.id == window.id)
        .map(|found| found.title)
        .unwrap_or_default()
}

#[test]
fn a_click_reaches_the_button_without_the_mouse() {
    let window = TestWindow::open("Botloft desktop test: click");
    let save = named(&window, "button", "Save");
    assert_eq!(
        act(window.id, &save.runtime_id, &Action::Click).expect("click"),
        Acted::Done
    );
    assert_eq!(title(&window), "Saved");
}

#[test]
fn typing_replaces_a_field_and_a_check_box_toggles() {
    let window = TestWindow::open("Botloft desktop test: type");
    let field = with_value(&window, "hello");
    let typed = Action::Type("Ana Lima".to_owned());
    assert_eq!(
        act(window.id, &field.runtime_id, &typed).expect("type"),
        Acted::Done
    );
    with_value(&window, "Ana Lima");

    let remember = named(&window, "check box", "Remember");
    assert_eq!(remember.checked, Some(true));
    act(window.id, &remember.runtime_id, &Action::Click).expect("toggle");
    assert_eq!(named(&window, "check box", "Remember").checked, Some(false));
}

#[test]
fn a_password_field_is_never_typed_in() {
    let window = TestWindow::open("Botloft desktop test: password");
    let field = read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.password)
        .expect("the password field");
    let typed = Action::Type("hunter2".to_owned());
    assert!(matches!(
        act(window.id, &field.runtime_id, &typed),
        Err(DesktopError::Password)
    ));
}

#[test]
fn an_option_is_chosen_and_a_long_list_scrolls() {
    let window = TestWindow::open("Botloft desktop test: choose");
    let size = read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.kind == "combo box")
        .expect("the combo box");
    let large = Action::Select("Large".to_owned());
    act(window.id, &size.runtime_id, &large).expect("choose");
    let chosen = read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.kind == "combo box")
        .expect("the combo box");
    assert_eq!(chosen.value.as_deref(), Some("Large"));
    let none = Action::Select("Huge".to_owned());
    assert!(matches!(
        act(window.id, &size.runtime_id, &none),
        Err(DesktopError::NoOption(_))
    ));

    let last = format!("Item {}", ITEMS - 1);
    let shown = |name: &str| {
        read(window.id)
            .expect("read")
            .iter()
            .any(|control| control.name == name)
    };
    assert!(!shown(&last), "the end of the list starts out of sight");
    let list = read(window.id)
        .expect("read")
        .into_iter()
        .find(|control| control.kind == "list")
        .expect("the list");
    act(window.id, &list.runtime_id, &Action::Scroll(Scroll::Bottom)).expect("scroll");
    assert!(shown(&last), "scrolled to its end");
}

#[test]
fn a_control_gone_from_the_window_is_said_so() {
    let window = TestWindow::open("Botloft desktop test: gone");
    let missing = [42, 42, 42];
    assert!(matches!(
        act(window.id, &missing, &Action::Click),
        Err(DesktopError::NotThere)
    ));
}
