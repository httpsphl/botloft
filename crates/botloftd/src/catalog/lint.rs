//! The shape every sheet must have (spec 26.2).

use botloft_core::validate;

use super::{SHEETS, Sheet};

/// Longest instructions a sheet may hold, in characters: room is left for
/// what the chief adds, inside the 8 000 of a suggestion (spec 26.4).
pub(super) const INSTRUCTIONS_MAX: usize = 4_000;
const SUMMARY_MAX: usize = 120;

/// The headings every sheet's instructions have, in this order. They sit
/// under the rules file's own "Instructions from the owner" heading.
pub(super) const SECTIONS: &[&str] = &[
    "### What you do",
    "### How you work",
    "### What you hand back",
    "### Ask the owner first",
    "### With the crew",
    "### Privacy",
];

/// What is wrong with a sheet, one line each; empty when it is fine.
pub(super) fn problems(id: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let valid_id = !id.is_empty()
        && id.len() <= 32
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid_id {
        found.push(format!("`{id}` is not a valid id ([a-z0-9-], at most 32)"));
    }
    let sheet = match Sheet::parse(text) {
        Ok(sheet) => sheet,
        Err(err) => {
            found.push(format!("does not read: {err}"));
            return found;
        }
    };
    if let Err(err) = validate::name("name", &sheet.name) {
        found.push(err.to_string());
    }
    if sheet.role.trim().is_empty() {
        found.push("role is empty".to_owned());
    }
    if let Err(err) = validate::role(&sheet.role) {
        found.push(err.to_string());
    }
    let summary = sheet.summary.trim();
    if summary.is_empty() || summary.contains('\n') || summary.chars().count() > SUMMARY_MAX {
        found.push(format!(
            "summary must be one line of at most {SUMMARY_MAX} characters"
        ));
    }
    if let Err(err) = validate::instructions(&sheet.instructions) {
        found.push(err.to_string());
    }
    let length = sheet.instructions.chars().count();
    if length > INSTRUCTIONS_MAX {
        found.push(format!(
            "instructions are {length} characters, at most {INSTRUCTIONS_MAX}"
        ));
    }
    let mut at = 0;
    for section in SECTIONS {
        match sheet.instructions[at..].find(section) {
            Some(found_at) => at += found_at + section.len(),
            None => found.push(format!("instructions lack `{section}` (in order)")),
        }
    }
    let text = format!(
        "{} {} {} {}",
        sheet.name, sheet.role, sheet.summary, sheet.instructions
    );
    if text.to_lowercase().contains("claude") || text.to_lowercase().contains("anthropic") {
        found.push("the text names the model or its maker".to_owned());
    }
    found
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn every_sheet_has_the_right_shape() {
        let mut report = Vec::new();
        for (id, text) in SHEETS {
            for problem in problems(id, text) {
                report.push(format!("{id}: {problem}"));
            }
        }
        assert!(report.is_empty(), "catalog sheets:\n{}", report.join("\n"));
    }

    #[test]
    fn the_list_is_the_folder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog");
        let on_disk: BTreeSet<String> = std::fs::read_dir(dir)
            .expect("catalog folder")
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter_map(|name| name.strip_suffix(".toml").map(str::to_owned))
            .collect();
        let listed: BTreeSet<String> = SHEETS.iter().map(|(id, _)| (*id).to_owned()).collect();
        assert_eq!(listed, on_disk, "SHEETS and catalog/*.toml differ");
        assert_eq!(listed.len(), SHEETS.len(), "an id is listed twice");
    }

    #[test]
    fn a_broken_sheet_is_reported() {
        let found = problems("Bad Id", "category = \"code\"");
        assert!(found.iter().any(|line| line.contains("valid id")));
        assert!(found.iter().any(|line| line.contains("does not read")));

        let sheet = r#"category = "code"
name = "Claude"
role = "x"
summary = "x"
model = "default"
effort = "default"
instructions = "no sections"
"#;
        let found = problems("ok", sheet);
        assert!(found.iter().any(|line| line.contains("lack")), "{found:?}");
        assert!(
            found.iter().any(|line| line.contains("names the model")),
            "{found:?}"
        );
    }
}
