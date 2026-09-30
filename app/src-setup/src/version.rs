//! How the installed Botloft compares with the one this setup carries.

use serde::Serialize;

/// What is already on this computer, next to this setup's version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Relation {
    /// Botloft is not installed.
    None,
    /// An older version: the setup updates it.
    Older,
    /// This same version.
    Same,
    /// A newer version: the setup only opens it.
    Newer,
}

/// `major.minor.patch`, ignoring a pre-release or build tag.
fn parse(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.trim().split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
    let found = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(found)
}

/// An installed version that cannot be read counts as older, so the setup
/// offers to update it rather than refuse.
pub fn relation(installed: Option<&str>, this: &str) -> Relation {
    let Some(installed) = installed else {
        return Relation::None;
    };
    match (parse(installed), parse(this)) {
        (Some(old), Some(new)) if old == new => Relation::Same,
        (Some(old), Some(new)) if old > new => Relation::Newer,
        _ => Relation::Older,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_by_number_not_text() {
        assert_eq!(relation(Some("0.9.0"), "0.10.0"), Relation::Older);
        assert_eq!(relation(Some("0.10.0"), "0.9.0"), Relation::Newer);
        assert_eq!(relation(Some("0.5.0"), "0.5.0"), Relation::Same);
        assert_eq!(relation(None, "0.5.0"), Relation::None);
    }

    #[test]
    fn ignores_tags_and_treats_the_unreadable_as_older() {
        assert_eq!(relation(Some("0.5.0-beta.1"), "0.5.0"), Relation::Same);
        assert_eq!(relation(Some(" 1.2.3 "), "1.2.3"), Relation::Same);
        assert_eq!(relation(Some("banana"), "0.5.0"), Relation::Older);
        assert_eq!(relation(Some("1.2"), "0.5.0"), Relation::Older);
        assert_eq!(relation(Some("1.2.3.4"), "0.5.0"), Relation::Older);
    }
}
