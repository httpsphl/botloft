//! A release's version, `0.22.0`, compared number by number (spec 42.11).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64, u64, u64);

impl Version {
    /// `0.22.0` or `v0.22.0`; anything after `-` or `+` in the last number
    /// is left out.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().trim_start_matches('v').split('.');
        let mut next = || -> Option<u64> { parts.next()?.split(['-', '+']).next()?.parse().ok() };
        let version = Self(next()?, next()?, next()?);
        parts.next().is_none().then_some(version)
    }

    /// This daemon's own.
    pub fn running() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).unwrap_or(Self(0, 0, 0))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_their_numbers() {
        let parse = |text| Version::parse(text).expect(text);
        assert!(parse("0.10.0") > parse("0.9.9"));
        assert!(parse("1.0.0") > parse("0.99.99"));
        assert_eq!(parse("v0.22.0"), parse("0.22.0"));
        assert_eq!(parse("0.22.0-rc1").to_string(), "0.22.0");
        assert_eq!(Version::parse("0.22"), None);
        assert_eq!(Version::parse("0.22.0.1"), None);
        assert_eq!(Version::parse("latest"), None);
        assert!(Version::running() > parse("0.0.1"));
    }
}
