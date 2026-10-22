//! How much a file edit changes (spec 8.2): the lines a `Write`, `Edit` or
//! `MultiEdit` call adds and removes, counted from its whole input before the
//! chat cuts it to size.

use serde_json::Value;

use crate::protocol::EditLines;

/// Most pairs of lines compared for one edit. Past it, every line left after
/// the common start and end counts as changed: an upper bound, never less.
const MAX_PAIRS: usize = 4_000_000;

/// The lines `name` adds and removes, from its input. `None` for other
/// tools, for an input without the texts (Codex and Antigravity send only the
/// path) and for an `Edit` that replaces every match, whose count of matches
/// the input does not say.
pub fn edit_lines(name: &str, input: &Value) -> Option<EditLines> {
    let mut lines = EditLines {
        added: 0,
        removed: 0,
    };
    match name {
        "Write" => lines.added = count(text(input, "content")?.lines().count()),
        "Edit" => replace(&mut lines, input)?,
        "MultiEdit" => {
            for edit in input.get("edits")?.as_array()? {
                replace(&mut lines, edit)?;
            }
        }
        _ => return None,
    }
    Some(lines)
}

fn text<'a>(input: &'a Value, name: &str) -> Option<&'a str> {
    input.get(name).and_then(Value::as_str)
}

fn count(lines: usize) -> u32 {
    u32::try_from(lines).unwrap_or(u32::MAX)
}

/// Adds one `old_string` → `new_string` replacement to `lines`.
fn replace(lines: &mut EditLines, edit: &Value) -> Option<()> {
    if edit.get("replace_all").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let old: Vec<&str> = text(edit, "old_string")?.lines().collect();
    let new: Vec<&str> = text(edit, "new_string")?.lines().collect();
    let (added, removed) = changed(&old, &new);
    lines.added = lines.added.saturating_add(count(added));
    lines.removed = lines.removed.saturating_add(count(removed));
    Some(())
}

/// Lines only in `new` and lines only in `old`: what is left of each once the
/// longest run of lines they share, in order, is taken out.
fn changed(old: &[&str], new: &[&str]) -> (usize, usize) {
    let start = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let (old, new) = (&old[start..], &new[start..]);
    let end = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let (old, new) = (&old[..old.len() - end], &new[..new.len() - end]);
    let shared = if old.len().saturating_mul(new.len()) > MAX_PAIRS {
        0
    } else {
        shared(old, new)
    };
    (new.len() - shared, old.len() - shared)
}

/// Length of the longest run of lines `a` and `b` share, in order, keeping
/// two rows of the table.
fn shared(a: &[&str], b: &[&str]) -> usize {
    let mut above = vec![0; b.len() + 1];
    let mut row = vec![0; b.len() + 1];
    for line in a {
        for (j, other) in b.iter().enumerate() {
            row[j + 1] = if line == other {
                above[j] + 1
            } else {
                above[j + 1].max(row[j])
            };
        }
        std::mem::swap(&mut above, &mut row);
    }
    above[b.len()]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn lines(added: u32, removed: u32) -> Option<EditLines> {
        Some(EditLines { added, removed })
    }

    #[test]
    fn an_edit_counts_the_lines_it_changes() {
        let edit = json!({
            "file_path": "C:\\work\\a.rs",
            "old_string": "fn a() {\n    one();\n    two();\n}",
            "new_string": "fn a() {\n    one();\n    three();\n    four();\n}",
        });
        assert_eq!(edit_lines("Edit", &edit), lines(2, 1));
        let grow = json!({ "old_string": "a", "new_string": "a\nb\n" });
        assert_eq!(edit_lines("Edit", &grow), lines(1, 0));
        let gone = json!({ "old_string": "a\r\nb\r\n", "new_string": "" });
        assert_eq!(edit_lines("Edit", &gone), lines(0, 2));
        // A line changed in place is one removed and one added.
        let word = json!({ "old_string": "let x = 1;", "new_string": "let x = 2;" });
        assert_eq!(edit_lines("Edit", &word), lines(1, 1));
    }

    #[test]
    fn lines_moved_around_count_only_once() {
        let edit = json!({ "old_string": "a\nb\nc\nd", "new_string": "b\na\nc\nd\ne" });
        assert_eq!(edit_lines("Edit", &edit), lines(2, 1));
    }

    #[test]
    fn a_multi_edit_adds_up_its_edits() {
        let input = json!({
            "file_path": "/w/a.md",
            "edits": [
                { "old_string": "a", "new_string": "a\nb" },
                { "old_string": "x\ny", "new_string": "z" },
            ],
        });
        assert_eq!(edit_lines("MultiEdit", &input), lines(2, 2));
    }

    #[test]
    fn a_write_adds_every_line_it_writes() {
        let input = json!({ "file_path": "/w/new.txt", "content": "one\ntwo\nthree\n" });
        assert_eq!(edit_lines("Write", &input), lines(3, 0));
        assert_eq!(edit_lines("Write", &json!({ "content": "" })), lines(0, 0));
    }

    #[test]
    fn no_counts_without_the_texts() {
        // Codex and Antigravity send only the path.
        assert_eq!(edit_lines("Edit", &json!({ "file_path": "/w/a" })), None);
        assert_eq!(edit_lines("Write", &json!({ "file_path": "/w/a" })), None);
        assert_eq!(
            edit_lines("Read", &json!({ "file_path": "/w/a", "content": "x" })),
            None
        );
        let every = json!({ "old_string": "a", "new_string": "b", "replace_all": true });
        assert_eq!(edit_lines("Edit", &every), None);
        let multi = json!({ "edits": [{ "old_string": "a", "new_string": "b" }, every] });
        assert_eq!(edit_lines("MultiEdit", &multi), None);
    }

    #[test]
    fn a_huge_edit_counts_every_changed_line() {
        let old: Vec<String> = (0..3000).map(|n| format!("old {n}")).collect();
        let mut new: Vec<String> = (0..3000).map(|n| format!("new {n}")).collect();
        new.insert(0, "same".into());
        new.push("end".into());
        let edit = json!({
            "old_string": format!("same\n{}\nend", old.join("\n")),
            "new_string": new.join("\n"),
        });
        // The common start and end are still found.
        assert_eq!(edit_lines("Edit", &edit), lines(3000, 3000));
    }
}
