//! HMAC-SHA-256 and HKDF-SHA-256 (RFC 2104, RFC 5869) over the `sha2` the
//! daemon already uses. The phone has both in its browser (WebCrypto), so the
//! two sides get the same bytes (spec 28.3).

use sha2::{Digest, Sha256};

const BLOCK: usize = 64;

/// `HMAC(key, parts[0] ‖ parts[1] ‖ ...)`.
pub fn hmac(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut padded = [0u8; BLOCK];
    if key.len() > BLOCK {
        padded[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        padded[..key.len()].copy_from_slice(key);
    }
    let mut inner = Sha256::new();
    inner.update(padded.map(|byte| byte ^ 0x36));
    for part in parts {
        inner.update(part);
    }
    let inner = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(padded.map(|byte| byte ^ 0x5c));
    outer.update(inner);
    outer.finalize().into()
}

/// `len` bytes (at most 255 × 32) from `ikm`, with the extract-then-expand of
/// HKDF. An empty `salt` is 32 zero bytes, as the RFC says.
pub fn hkdf(ikm: &[u8], salt: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    let zeros = [0u8; 32];
    let salt = if salt.is_empty() { &zeros[..] } else { salt };
    let prk = hmac(salt, &[ikm]);
    let mut okm = Vec::with_capacity(len);
    let mut block: Vec<u8> = Vec::new();
    let mut counter = 1u8;
    while okm.len() < len {
        block = hmac(&prk, &[&block, info, &[counter]]).to_vec();
        okm.extend_from_slice(&block);
        counter += 1;
    }
    okm.truncate(len);
    okm
}

/// Whether two byte strings are equal, without stopping at the first
/// difference.
pub fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |diff, (x, y)| diff | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(text: &str) -> Vec<u8> {
        hex::decode(text).expect("hex")
    }

    #[test]
    fn hmac_matches_rfc_4231() {
        // Test case 1, and case 6 (a key longer than a block).
        let key = [0x0b; 20];
        assert_eq!(
            hex::encode(hmac(&key, &[b"Hi There"])),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        let long = [0xaa; 131];
        assert_eq!(
            hex::encode(hmac(
                &long,
                &[b"Test Using Larger Than Block-Size Key - Hash Key First"]
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
        // Parts are just one message cut in pieces.
        assert_eq!(hmac(&key, &[b"Hi ", b"There"]), hmac(&key, &[b"Hi There"]));
    }

    #[test]
    fn hkdf_matches_rfc_5869() {
        let okm = hkdf(
            &[0x0b; 22],
            &unhex("000102030405060708090a0b0c"),
            &unhex("f0f1f2f3f4f5f6f7f8f9"),
            42,
        );
        assert_eq!(
            hex::encode(okm),
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        );
        // Case 3: no salt and no info.
        let okm = hkdf(&[0x0b; 22], &[], &[], 42);
        assert_eq!(
            hex::encode(okm),
            "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"
        );
    }

    #[test]
    fn same_tells_equal_from_different() {
        assert!(same(b"abc", b"abc"));
        assert!(!same(b"abc", b"abd"));
        assert!(!same(b"abc", b"ab"));
    }
}
