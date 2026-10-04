//! A window read as text (spec 24.5): one control a line, indented by its
//! place in the window, with a `ref` the bot names it by, its kind, name,
//! value and state. A password field never shows its value.

/// The most lines one reading gives; `from` goes on from where it stopped.
pub const LINES_MAX: usize = 600;
/// The longest value shown, in characters.
const VALUE_MAX: usize = 200;

/// A control of a window, as the accessibility tree has it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Control {
    /// How deep it sits in the window: 0 for the window's own children.
    pub depth: usize,
    /// Its kind, in words: "button", "edit", "check box".
    pub kind: &'static str,
    pub name: String,
    /// What it holds (a field's text, a cell's value), when it has a value.
    pub value: Option<String>,
    /// A password field: its value is never read.
    pub password: bool,
    pub enabled: bool,
    /// For a check box or toggle button.
    pub checked: Option<bool>,
    /// For what opens and closes: a combo box, a tree item, a menu.
    pub expanded: Option<bool>,
    pub selected: bool,
    /// Windows' id for it, to find it again to act on it (D2).
    pub runtime_id: Vec<i32>,
}

/// The `ref` of the control at `index` in a reading.
pub fn reference(index: usize) -> String {
    format!("d{}", index + 1)
}

/// The control a `ref` names in `controls`, if it is one.
pub fn by_reference<'a>(controls: &'a [Control], reference: &str) -> Option<&'a Control> {
    let number: usize = reference.strip_prefix('d')?.parse().ok()?;
    controls.get(number.checked_sub(1)?)
}

fn clip(text: &str) -> String {
    let line: String = text.replace(['\r', '\n'], " ");
    if line.chars().count() <= VALUE_MAX {
        return line;
    }
    let cut: String = line.chars().take(VALUE_MAX).collect();
    format!("{cut}…")
}

fn line(index: usize, control: &Control) -> String {
    let mut text = format!(
        "{}[{}] {}",
        "  ".repeat(control.depth),
        reference(index),
        control.kind
    );
    if !control.name.is_empty() {
        text.push_str(&format!(" \"{}\"", clip(&control.name)));
    }
    if control.password {
        text.push_str(" (password: the owner types it, you never read or type it)");
    } else if let Some(value) = &control.value {
        text.push_str(&format!(" = \"{}\"", clip(value)));
    }
    let mut states = Vec::new();
    match control.checked {
        Some(true) => states.push("checked"),
        Some(false) => states.push("not checked"),
        None => {}
    }
    match control.expanded {
        Some(true) => states.push("open"),
        Some(false) => states.push("closed"),
        None => {}
    }
    if control.selected {
        states.push("selected");
    }
    if !control.enabled {
        states.push("disabled");
    }
    if !states.is_empty() {
        text.push_str(&format!(" ({})", states.join(", ")));
    }
    text
}

/// The window's controls as text, from line `from`, at most
/// [`LINES_MAX`] lines; and where the next reading goes on, when it stopped
/// short.
pub fn render(controls: &[Control], from: usize) -> (String, Option<usize>) {
    let end = controls.len().min(from.saturating_add(LINES_MAX));
    let lines: Vec<String> = controls
        .iter()
        .enumerate()
        .skip(from)
        .take(end.saturating_sub(from))
        .map(|(index, control)| line(index, control))
        .collect();
    let next = (end < controls.len()).then_some(end);
    (lines.join("\n"), next)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control(kind: &'static str, name: &str) -> Control {
        Control {
            kind,
            name: name.to_owned(),
            enabled: true,
            ..Control::default()
        }
    }

    #[test]
    fn each_control_is_a_line_with_its_ref_kind_name_value_and_state() {
        let controls = vec![
            Control {
                value: Some("Ana\nLima".to_owned()),
                ..control("edit", "Name")
            },
            Control {
                depth: 1,
                checked: Some(true),
                ..control("check box", "Remember me")
            },
            Control {
                enabled: false,
                ..control("button", "Save")
            },
        ];
        let (text, next) = render(&controls, 0);
        assert_eq!(
            text,
            "[d1] edit \"Name\" = \"Ana Lima\"\n  [d2] check box \"Remember me\" (checked)\n\
             [d3] button \"Save\" (disabled)"
        );
        assert_eq!(next, None);
        assert_eq!(
            by_reference(&controls, "d2").map(|found| found.kind),
            Some("check box")
        );
        assert!(by_reference(&controls, "d0").is_none());
        assert!(by_reference(&controls, "x2").is_none());
    }

    #[test]
    fn a_password_field_never_shows_its_value() {
        let field = Control {
            password: true,
            value: Some("hunter2".to_owned()),
            ..control("edit", "Password")
        };
        let (text, _) = render(&[field], 0);
        assert!(!text.contains("hunter2"), "{text}");
        assert!(text.contains("the owner types it"), "{text}");
    }

    #[test]
    fn a_long_window_goes_on_from_where_it_stopped() {
        let controls: Vec<Control> = (0..LINES_MAX + 5)
            .map(|_| control("list item", "Row"))
            .collect();
        let (first, next) = render(&controls, 0);
        assert_eq!(first.lines().count(), LINES_MAX);
        assert_eq!(next, Some(LINES_MAX));
        let (rest, next) = render(&controls, LINES_MAX);
        assert_eq!(rest.lines().count(), 5);
        assert!(rest.starts_with(&format!("[d{}]", LINES_MAX + 1)));
        assert_eq!(next, None);
    }
}
