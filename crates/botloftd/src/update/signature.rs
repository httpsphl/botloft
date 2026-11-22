//! The updater's signature on a server download (spec 42.11): the release
//! signs `botloftd-linux-<arch>.tar.gz` with the key that signs the app's
//! updates (15.5), as a minisign signature in base64 (`.sig`), and the
//! daemon checks it before anything is replaced.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use minisign_verify::{PublicKey, Signature};

/// The updater's public key: the second line of the minisign key that
/// `app/src-tauri/tauri.conf.json` holds in base64.
pub const UPDATE_KEY: &str = "RWRQcgifdFU5+Ll55Oql5sp7uCDg4qR2lqbiuBVqSBktdDSEdXkSmqBo";

/// Whether `sig` (a `.sig` file as the release writes it) signs `bytes`
/// with the key whose base64 line is `key`.
pub fn signed(key: &str, bytes: &[u8], sig: &str) -> Result<(), String> {
    let key = PublicKey::from_base64(key).map_err(|err| format!("the key: {err}"))?;
    let text = BASE64
        .decode(sig.trim())
        .ok()
        .and_then(|text| String::from_utf8(text).ok())
        .ok_or("the signature is not base64 text")?;
    let signature = Signature::decode(&text).map_err(|err| format!("the signature: {err}"))?;
    key.verify(bytes, &signature, false)
        .map_err(|_| "the signature does not match: nothing was replaced".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway key, made for these tests with `tauri signer generate`.
    const TEST_KEY: &str = "RWTy2bgxLVDVog3+kEthY0nlGPCA3fWl4Qxta5G5x0xeTlXhD4VGJCYz";
    const PAYLOAD: &[u8] = b"botloftd test bytes\n";
    const SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUeTJiZ3hMVkRWb2o5djI2NU5UTDJBdW5pSUpjNUdON1VtblFZbzNxeEY4MVhjeExkUVVOWTJUMGlaOUREK2N2N2lLM0doWmJwRThWZ2J0eTJtWGw0UXpsUlo0QVlhSlFjPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1MzM2Njk2CWZpbGU6cGF5bG9hZC5iaW4KeGR0Ly9Vd1dHMVFhWWVRcmxlNnNsS3d1SU1SOWNCT05NeXdtYVlTL09zWEJ1bFdCUUlCZ0N1UkhUR1VaMVR4RVB5ZWF0c0VBRGV3ZEJlc2k1SEJTQnc9PQo=";

    #[test]
    fn a_signed_download_passes_and_a_changed_one_does_not() {
        assert_eq!(signed(TEST_KEY, PAYLOAD, SIG), Ok(()));
        assert!(signed(TEST_KEY, b"botloftd test bytes!\n", SIG).is_err());
        assert!(signed(UPDATE_KEY, PAYLOAD, SIG).is_err(), "another key");
        assert!(signed(TEST_KEY, PAYLOAD, "not base64").is_err());
    }
}
