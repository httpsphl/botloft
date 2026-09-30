//! Which instruction files Claude Code loads into a bot (spec 7.5).
//!
//! Claude Code reads `CLAUDE.md` and its relatives from the working directory
//! and from every directory above it. A bot's folder sits inside the owner's
//! profile by default, so the owner's personal `.claude\CLAUDE.md` would reach
//! every bot. `claudeMdExcludes` skips what is above the crew's folder.

use std::path::Path;

/// What Claude Code looks for in each directory above the working directory.
const INSTRUCTION_FILES: [&str; 6] = [
    "CLAUDE.md",
    "CLAUDE.local.md",
    "AGENTS.md",
    ".claude/CLAUDE.md",
    ".claude/AGENTS.md",
    ".claude/rules/**",
];

/// `claudeMdExcludes` patterns for every instruction file in the folders
/// above `crew_dir`. The crew's folder, the bots' folders in it and the work
/// folder are left alone: what is there is for the bots.
pub fn excludes_above(crew_dir: &Path) -> Vec<String> {
    crew_dir
        .ancestors()
        .skip(1)
        .flat_map(|dir| {
            let dir = glob_dir(dir);
            INSTRUCTION_FILES
                .iter()
                .map(move |file| format!("{dir}/{file}"))
        })
        .collect()
}

/// A directory in the form Claude Code matches the patterns against on
/// Windows: the path it was started with, letter case included, with either
/// separator (`C:/Users/ana`). The `//c/...` form of permission rules does
/// not match here (spec 19). Brackets, braces and parentheses in a folder
/// name would be read as glob syntax, so they become `?`, any one character.
fn glob_dir(dir: &Path) -> String {
    let text = dir.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text);
    text.trim_end_matches('/')
        .chars()
        .map(|c| match c {
            '[' | ']' | '{' | '}' | '(' | ')' | '*' | '?' => '?',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_folder_above_the_crew_is_covered_and_nothing_below() {
        let patterns = excludes_above(Path::new("/ws/site"));
        assert_eq!(
            patterns,
            [
                "/ws/CLAUDE.md",
                "/ws/CLAUDE.local.md",
                "/ws/AGENTS.md",
                "/ws/.claude/CLAUDE.md",
                "/ws/.claude/AGENTS.md",
                "/ws/.claude/rules/**",
                "/CLAUDE.md",
                "/CLAUDE.local.md",
                "/AGENTS.md",
                "/.claude/CLAUDE.md",
                "/.claude/AGENTS.md",
                "/.claude/rules/**",
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_keep_the_drive_and_case_and_use_forward_slashes() {
        let patterns = excludes_above(Path::new(r"C:\Users\Ana\Botloft\site"));
        assert_eq!(patterns.len(), 4 * INSTRUCTION_FILES.len());
        for expected in [
            "C:/Users/Ana/Botloft/CLAUDE.md",
            "C:/Users/Ana/.claude/CLAUDE.md",
            "C:/Users/Ana/.claude/rules/**",
            "C:/Users/CLAUDE.local.md",
            "C:/AGENTS.md",
        ] {
            assert!(patterns.iter().any(|p| p == expected), "{expected} missing");
        }
        // The bot's own memory and rules, the crew's folder and its work
        // folder must keep loading.
        assert!(
            patterns
                .iter()
                .all(|p| !p.starts_with("C:/Users/Ana/Botloft/site"))
        );

        let verbatim = excludes_above(Path::new(r"\\?\D:\bots\site"));
        assert_eq!(verbatim[0], "D:/bots/CLAUDE.md");
        assert_eq!(
            verbatim.last().map(String::as_str),
            Some("D:/.claude/rules/**")
        );
    }

    #[test]
    fn glob_syntax_in_a_folder_name_matches_as_any_character() {
        let patterns = excludes_above(Path::new("/home/ana (work) [x] {y}/Botloft/site"));
        assert!(
            patterns
                .iter()
                .any(|p| p == "/home/ana ?work? ?x? ?y?/.claude/rules/**"),
            "{patterns:?}"
        );
        // Characters a glob reads literally stay as they are.
        assert_eq!(
            glob_dir(Path::new("/a!b+c@d#e$f, g;h=i~j é")),
            "/a!b+c@d#e$f, g;h=i~j é"
        );
    }
}
