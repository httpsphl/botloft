//! Tokens (spec 13). The owner token lives in `secrets\owner.token`,
//! readable only by the current user; the daemon keeps just its SHA-256.

use std::fmt;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::platform;

const OWNER_TOKEN_FILE: &str = "owner.token";
const TOKEN_BYTES: usize = 32;

/// 32 random bytes as 64 lowercase hex characters.
pub fn random_token() -> io::Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|err| io::Error::other(err.to_string()))?;
    Ok(hex::encode(bytes))
}

/// SHA-256 of a token. Comparing hashes keeps the check independent of how
/// many leading characters of a guess are right.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenHash([u8; 32]);

impl TokenHash {
    pub fn of(token: &str) -> Self {
        Self(Sha256::digest(token.as_bytes()).into())
    }

    pub fn matches(&self, candidate: &str) -> bool {
        let other = Self::of(candidate);
        self.0
            .iter()
            .zip(other.0.iter())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Debug for TokenHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TokenHash(<redacted>)")
    }
}

/// Reads the owner token, creating it (and the secrets folder) on first run.
/// Both are restricted to the current user every time.
pub fn load_or_create_owner_token(secrets_dir: &Path) -> io::Result<TokenHash> {
    std::fs::create_dir_all(secrets_dir)?;
    platform::restrict_to_current_user(secrets_dir)?;
    let path = secrets_dir.join(OWNER_TOKEN_FILE);
    let token = match std::fs::read_to_string(&path) {
        Ok(text) if is_valid_token(text.trim()) => text.trim().to_owned(),
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} is not a valid token; delete it to create a new one",
                    path.display()
                ),
            ));
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let token = random_token()?;
            let tmp = path.with_extension("tmp");
            std::fs::write(&tmp, &token)?;
            std::fs::rename(&tmp, &path)?;
            token
        }
        Err(err) => return Err(err),
    };
    platform::restrict_to_current_user(&path)?;
    Ok(TokenHash::of(&token))
}

fn is_valid_token(text: &str) -> bool {
    text.len() == TOKEN_BYTES * 2 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_tokens_are_hex_and_distinct() {
        let a = random_token().expect("token");
        let b = random_token().expect("token");
        assert!(is_valid_token(&a));
        assert_ne!(a, b);
    }

    #[test]
    fn hash_matches_only_the_same_token() {
        let hash = TokenHash::of("secret");
        assert!(hash.matches("secret"));
        assert!(!hash.matches("secreT"));
        assert!(!hash.matches(""));
        assert_eq!(format!("{hash:?}"), "TokenHash(<redacted>)");
    }

    #[test]
    fn owner_token_is_created_once_and_reused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let secrets = dir.path().join("secrets");
        let first = load_or_create_owner_token(&secrets).expect("create");
        let token = std::fs::read_to_string(secrets.join(OWNER_TOKEN_FILE)).expect("read");
        assert!(first.matches(&token));
        let second = load_or_create_owner_token(&secrets).expect("reuse");
        assert_eq!(first, second);
    }

    #[test]
    fn a_corrupt_token_file_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(OWNER_TOKEN_FILE), "not-a-token").expect("write");
        assert!(load_or_create_owner_token(dir.path()).is_err());
    }
}
