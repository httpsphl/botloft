//! Secrets the server hands out: 32 random bytes, kept only as a SHA-256.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::error::ApiError;

/// A new secret, as it goes to whoever must hold it.
pub fn random() -> Result<String, ApiError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|err| {
        tracing::error!("random bytes: {err}");
        ApiError::internal()
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// What the database keeps of a secret.
pub fn hash(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_differ_and_hash_the_same_way() {
        let (a, b) = (random().expect("a"), random().expect("b"));
        assert_ne!(a, b);
        assert_eq!(a.len(), 43);
        assert_eq!(hash(&a), hash(&a));
        assert_ne!(hash(&a), hash(&b));
        assert_eq!(hash(&a).len(), 64);
    }
}
