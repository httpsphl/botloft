//! Folder-safe slugs and bot handles derived from display names.
//!
//! Slugs name workspace folders and never change after creation (spec 5).
//! Handles address bots inside a crew and follow the bot's current name.

/// Longest slug or handle, in characters. Short folder names keep workspace
/// paths well under the Windows 260-character limit.
pub const MAX_LEN: usize = 32;

/// Device names Windows refuses as file or folder names.
const WINDOWS_RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Turns a display name into lowercase ASCII letters, digits and single
/// hyphens, transliterating accents (`Revisão` -> `revisao`).
///
/// Returns `fallback` when nothing usable is left and appends it to names
/// Windows reserves for devices (`con` -> `con-bot`).
pub fn slugify(name: &str, fallback: &str) -> String {
    let ascii = deunicode::deunicode(name);
    let mut out = String::with_capacity(ascii.len().min(MAX_LEN));
    let mut hyphen_pending = false;
    for ch in ascii.chars() {
        if ch.is_ascii_alphanumeric() {
            if hyphen_pending && !out.is_empty() {
                out.push('-');
            }
            hyphen_pending = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            hyphen_pending = true;
        }
    }
    let mut out = truncate(&out, MAX_LEN);
    if out.is_empty() {
        return fallback.to_owned();
    }
    if WINDOWS_RESERVED.contains(&out.as_str()) {
        out = format!("{out}-{fallback}");
    }
    out
}

/// Returns `base` if `taken` rejects it, otherwise the first free `base-2`,
/// `base-3`, ... shortened so the result still fits in [`MAX_LEN`].
pub fn unique(base: &str, mut taken: impl FnMut(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_owned();
    }
    let mut n: u32 = 2;
    loop {
        let suffix = format!("-{n}");
        let candidate = format!("{}{suffix}", truncate(base, MAX_LEN - suffix.len()));
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Longest attachment file name kept, in characters.
const FILE_NAME_MAX: usize = 100;

/// A file name that is safe to write in a folder on Windows: no folders,
/// no reserved characters or device names, no trailing dots or spaces
/// (spec 9.5). Keeps the extension when it has to shorten the name.
pub fn file_name(name: &str) -> String {
    let last = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let cleaned: String = last
        .chars()
        .filter(|ch| !ch.is_control() && !r#"<>:"/\|?*"#.contains(*ch))
        .collect();
    let cleaned = cleaned
        .trim()
        .trim_end_matches(['.', ' '])
        .trim_start_matches('.');
    let mut name = if cleaned.is_empty() {
        "file".to_owned()
    } else {
        cleaned.to_owned()
    };
    let (stem, extension) = match name.rfind('.') {
        Some(dot) if dot > 0 => (name[..dot].to_owned(), name[dot..].to_owned()),
        _ => (name.clone(), String::new()),
    };
    if WINDOWS_RESERVED.contains(&stem.to_ascii_lowercase().as_str()) {
        name = format!("{stem}-file{extension}");
    }
    if name.chars().count() > FILE_NAME_MAX {
        let (stem, extension) = match name.rfind('.') {
            Some(dot) if dot > 0 && name.len() - dot <= 12 => (&name[..dot], &name[dot..]),
            _ => (name.as_str(), ""),
        };
        let keep = FILE_NAME_MAX.saturating_sub(extension.chars().count());
        name = format!("{}{extension}", stem.chars().take(keep).collect::<String>());
    }
    name
}

/// Cuts an ASCII slug to `max` characters without leaving a trailing hyphen.
fn truncate(slug: &str, max: usize) -> String {
    let cut = &slug[..slug.len().min(max)];
    cut.trim_end_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_and_collapses_separators() {
        assert_eq!(slugify("Revisão de Código", "bot"), "revisao-de-codigo");
        assert_eq!(slugify("  Site -- Loja!! ", "crew"), "site-loja");
        assert_eq!(slugify("QA_Bot 2", "bot"), "qa-bot-2");
    }

    #[test]
    fn falls_back_when_nothing_is_left() {
        assert_eq!(slugify("!!!", "bot"), "bot");
        assert_eq!(slugify("", "crew"), "crew");
    }

    #[test]
    fn avoids_windows_device_names() {
        assert_eq!(slugify("CON", "bot"), "con-bot");
        assert_eq!(slugify("lpt1", "crew"), "lpt1-crew");
        assert_eq!(slugify("console", "bot"), "console");
    }

    #[test]
    fn limits_length_without_trailing_hyphen() {
        let slug = slugify("a very long display name for a bot that keeps going", "bot");
        assert!(slug.len() <= MAX_LEN, "{slug}");
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn unique_appends_the_first_free_counter() {
        let taken = ["docs", "docs-2"];
        assert_eq!(unique("docs", |s| taken.contains(&s)), "docs-3");
        assert_eq!(unique("free", |s| taken.contains(&s)), "free");
    }

    #[test]
    fn unique_keeps_suffixed_slugs_within_the_limit() {
        let base = "x".repeat(MAX_LEN);
        let slug = unique(&base, |s| s == base);
        assert_eq!(slug.len(), MAX_LEN);
        assert!(slug.ends_with("-2"));
    }

    #[test]
    fn file_names_lose_folders_and_forbidden_characters() {
        assert_eq!(file_name("relatório final.pdf"), "relatório final.pdf");
        assert_eq!(file_name(r"C:\Users\ana\notes.txt"), "notes.txt");
        assert_eq!(file_name("../../etc/passwd"), "passwd");
        assert_eq!(file_name("a<b>:c?.png"), "abc.png");
        assert_eq!(file_name("trailing. . "), "trailing");
        assert_eq!(file_name(".hidden"), "hidden");
        assert_eq!(file_name(""), "file");
        assert_eq!(file_name("..."), "file");
        assert_eq!(file_name("CON.txt"), "CON-file.txt");
        assert_eq!(file_name("nul"), "nul-file");
    }

    #[test]
    fn long_file_names_keep_their_extension() {
        let name = format!("{}.xlsx", "n".repeat(300));
        let short = file_name(&name);
        assert_eq!(short.chars().count(), FILE_NAME_MAX);
        assert!(short.ends_with(".xlsx"));
    }
}
