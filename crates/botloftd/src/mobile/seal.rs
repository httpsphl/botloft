//! Sealing what crosses the relay (spec 28.3): ECDH P-256 for the keys,
//! HKDF-SHA-256 to turn them into one AES-256-GCM key per direction, and
//! HMAC-SHA-256 for the proofs of the QR code. All of it exists in the
//! phone's browser, and `docs/test-vectors/mobile.json` holds bytes both
//! sides must agree on.

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use p256::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};

use super::kdf::{hkdf, hmac};

/// The version of the sealed message (`v`).
pub const VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SealError {
    #[error("not a valid key")]
    Key,
    #[error("not a sealed message")]
    Format,
    #[error("the message does not open")]
    Auth,
    #[error("no random bytes")]
    Random,
}

/// One side's key for the exchange.
pub struct Keypair {
    secret: SecretKey,
    public: Vec<u8>,
}

impl Keypair {
    pub fn generate() -> Result<Self, SealError> {
        loop {
            let mut bytes = [0u8; 32];
            getrandom::fill(&mut bytes).map_err(|_| SealError::Random)?;
            // Not every 32 bytes are a scalar below the curve's order; try again.
            if let Ok(pair) = Self::from_bytes(&bytes) {
                return Ok(pair);
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SealError> {
        let secret = SecretKey::from_slice(bytes).map_err(|_| SealError::Key)?;
        let public = secret.public_key().to_sec1_bytes().to_vec();
        Ok(Self { secret, public })
    }

    /// The point, uncompressed (65 bytes), as WebCrypto exports it.
    pub fn public(&self) -> &[u8] {
        &self.public
    }

    fn shared(&self, theirs: &[u8]) -> Result<Vec<u8>, SealError> {
        let theirs = PublicKey::from_sec1_bytes(theirs).map_err(|_| SealError::Key)?;
        let shared =
            p256::ecdh::diffie_hellman(self.secret.to_nonzero_scalar(), theirs.as_affine());
        Ok(shared.raw_secret_bytes().to_vec())
    }
}

/// The two keys of a connection, one per direction.
#[derive(Clone, PartialEq, Eq)]
pub struct Keys {
    pub c2p: [u8; 32],
    pub p2c: [u8; 32],
}

/// Both sides derive the same pair: the shared point, salted with the QR's
/// secret, which the server never saw.
pub fn derive(mine: &Keypair, theirs: &[u8], secret: &[u8]) -> Result<Keys, SealError> {
    let okm = hkdf(&mine.shared(theirs)?, secret, b"botloft-mobile-1", 64);
    let mut keys = Keys {
        c2p: [0; 32],
        p2c: [0; 32],
    };
    keys.c2p.copy_from_slice(&okm[..32]);
    keys.p2c.copy_from_slice(&okm[32..]);
    Ok(keys)
}

/// What the phone proves when it joins: that it holds the QR's secret.
pub fn join_proof(secret: &[u8], id: &str, phone_pub: &[u8]) -> [u8; 32] {
    hmac(secret, &[b"botloft-pair-1", id.as_bytes(), phone_pub])
}

/// What the computer proves when it accepts.
pub fn accept_proof(secret: &[u8], id: &str, daemon_pub: &[u8], phone_pub: &[u8]) -> [u8; 32] {
    hmac(
        secret,
        &[
            b"botloft-pair-1-accept",
            id.as_bytes(),
            daemon_pub,
            phone_pub,
        ],
    )
}

/// Six digits both screens show, so the owner sees it is the same pairing.
pub fn code(secret: &[u8], daemon_pub: &[u8], phone_pub: &[u8]) -> String {
    let ikm = [daemon_pub, phone_pub].concat();
    let bytes = hkdf(&ikm, secret, b"botloft-pair-1-code", 3);
    let bits = (u32::from(bytes[0]) << 12) | (u32::from(bytes[1]) << 4) | u32::from(bytes[2] >> 4);
    format!("{:06}", bits % 1_000_000)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    ComputerToPhone = 1,
    PhoneToComputer = 2,
}

#[derive(Serialize, Deserialize)]
struct Sealed {
    v: u8,
    seq: u64,
    ct: String,
}

fn aad(dir: Dir, device: &str, seq: u64) -> Vec<u8> {
    let mut aad = vec![VERSION, dir as u8];
    aad.extend_from_slice(device.as_bytes());
    aad.extend_from_slice(&seq.to_be_bytes());
    aad
}

fn nonce(seq: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&seq.to_be_bytes());
    nonce
}

fn cipher(key: &[u8; 32]) -> Result<Aes256Gcm, SealError> {
    Aes256Gcm::new_from_slice(key).map_err(|_| SealError::Key)
}

/// The text the relay carries: `{"v":1,"seq":N,"ct":"..."}`. The `seq` must
/// never repeat under a key, so it comes from a counter that is saved first.
pub fn seal(
    key: &[u8; 32],
    dir: Dir,
    device: &str,
    seq: u64,
    plain: &[u8],
) -> Result<String, SealError> {
    let nonce = nonce(seq);
    let ct = cipher(key)?
        .encrypt(
            &nonce.into(),
            Payload {
                msg: plain,
                aad: &aad(dir, device, seq),
            },
        )
        .map_err(|_| SealError::Auth)?;
    serde_json::to_string(&Sealed {
        v: VERSION,
        seq,
        ct: B64.encode(ct),
    })
    .map_err(|_| SealError::Format)
}

/// The `seq` and the plain text of a sealed message, if it opens.
pub fn open(
    key: &[u8; 32],
    dir: Dir,
    device: &str,
    body: &str,
) -> Result<(u64, Vec<u8>), SealError> {
    let sealed: Sealed = serde_json::from_str(body).map_err(|_| SealError::Format)?;
    if sealed.v != VERSION {
        return Err(SealError::Format);
    }
    let ct = B64.decode(&sealed.ct).map_err(|_| SealError::Format)?;
    let plain = cipher(key)?
        .decrypt(
            &nonce(sealed.seq).into(),
            Payload {
                msg: &ct,
                aad: &aad(dir, device, sealed.seq),
            },
        )
        .map_err(|_| SealError::Auth)?;
    Ok((sealed.seq, plain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const SECRET: [u8; 32] = [0x33; 32];
    const ID: &str = "AAAAAAAAAAAAAAAAAAAAAA";

    fn pair() -> (Keypair, Keypair) {
        (
            Keypair::from_bytes(&[0x11; 32]).expect("daemon key"),
            Keypair::from_bytes(&[0x22; 32]).expect("phone key"),
        )
    }

    #[test]
    fn both_sides_derive_the_same_keys_and_the_secret_matters() {
        let (daemon, phone) = pair();
        let a = derive(&daemon, phone.public(), &SECRET).expect("daemon side");
        let b = derive(&phone, daemon.public(), &SECRET).expect("phone side");
        assert!(a == b);
        assert_ne!(a.c2p, a.p2c);
        // The server knows both public keys but not the secret: not the keys.
        let guess = derive(&daemon, phone.public(), &[0x34; 32]).expect("wrong secret");
        assert!(guess != a);
        assert_eq!(daemon.public().len(), 65);
        assert_eq!(daemon.public()[0], 4);
        assert_eq!(
            derive(&daemon, &[1, 2, 3], &SECRET).err(),
            Some(SealError::Key)
        );
    }

    #[test]
    fn a_sealed_message_opens_only_for_its_key_direction_device_and_seq() {
        let key = [7u8; 32];
        let body = seal(&key, Dir::ComputerToPhone, "dev_a", 5, b"hello").expect("seal");
        assert_eq!(
            open(&key, Dir::ComputerToPhone, "dev_a", &body).expect("open"),
            (5, b"hello".to_vec())
        );
        assert_eq!(
            open(&[8u8; 32], Dir::ComputerToPhone, "dev_a", &body).err(),
            Some(SealError::Auth)
        );
        assert_eq!(
            open(&key, Dir::PhoneToComputer, "dev_a", &body).err(),
            Some(SealError::Auth)
        );
        assert_eq!(
            open(&key, Dir::ComputerToPhone, "dev_b", &body).err(),
            Some(SealError::Auth)
        );
        // Changing the counter, or a byte of the text, breaks it.
        let moved = body.replace("\"seq\":5", "\"seq\":6");
        assert_eq!(
            open(&key, Dir::ComputerToPhone, "dev_a", &moved).err(),
            Some(SealError::Auth)
        );
        let mut value: Value = serde_json::from_str(&body).expect("json");
        let ct = value["ct"].as_str().expect("ct").to_owned();
        let flipped = format!(
            "{}{}",
            &ct[..ct.len() - 1],
            if ct.ends_with('A') { 'B' } else { 'A' }
        );
        value["ct"] = json!(flipped);
        assert_eq!(
            open(&key, Dir::ComputerToPhone, "dev_a", &value.to_string()).err(),
            Some(SealError::Auth)
        );
        for bad in [
            "",
            "not json",
            "{\"v\":2,\"seq\":1,\"ct\":\"AA\"}",
            "{\"v\":1,\"seq\":1,\"ct\":\"***\"}",
        ] {
            assert_eq!(
                open(&key, Dir::ComputerToPhone, "dev_a", bad).err(),
                Some(SealError::Format),
                "{bad}"
            );
        }
    }

    #[test]
    fn the_code_is_six_digits_and_follows_the_keys() {
        let (daemon, phone) = pair();
        let code_a = code(&SECRET, daemon.public(), phone.public());
        assert_eq!(code_a.len(), 6);
        assert!(code_a.bytes().all(|b| b.is_ascii_digit()));
        assert_ne!(code_a, code(&SECRET, phone.public(), daemon.public()));
        assert_ne!(
            join_proof(&SECRET, ID, phone.public()),
            join_proof(&[0x34; 32], ID, phone.public())
        );
        assert_ne!(
            join_proof(&SECRET, ID, phone.public()),
            accept_proof(&SECRET, ID, daemon.public(), phone.public())
        );
    }

    /// The bytes the phone's code must reproduce. Run with
    /// `BOTLOFT_WRITE_VECTORS=1` to write the file again.
    #[test]
    fn the_test_vectors_are_what_the_code_makes() {
        let (daemon, phone) = pair();
        let keys = derive(&daemon, phone.public(), &SECRET).expect("keys");
        let b64 = |bytes: &[u8]| B64.encode(bytes);
        let c2p = seal(
            &keys.c2p,
            Dir::ComputerToPhone,
            "dev_phone",
            1,
            br#"{"t":"sync"}"#,
        )
        .expect("c2p");
        let p2c = seal(
            &keys.p2c,
            Dir::PhoneToComputer,
            "dev_phone",
            7,
            "olá 🔥".as_bytes(),
        )
        .expect("p2c");
        let vectors = json!({
            "about": "Spec 28.3. Generated by botloftd (mobile::seal tests); the PWA must reproduce every value.",
            "pair": {
                "id": ID,
                "secret": b64(&SECRET),
                "daemonPrivateHex": hex::encode([0x11u8; 32]),
                "phonePrivateHex": hex::encode([0x22u8; 32]),
                "daemonPublic": b64(daemon.public()),
                "phonePublic": b64(phone.public()),
                "joinProof": b64(&join_proof(&SECRET, ID, phone.public())),
                "acceptProof": b64(&accept_proof(&SECRET, ID, daemon.public(), phone.public())),
                "code": code(&SECRET, daemon.public(), phone.public()),
                "computerToPhoneKeyHex": hex::encode(keys.c2p),
                "phoneToComputerKeyHex": hex::encode(keys.p2c),
            },
            "seal": [
                {
                    "direction": "computerToPhone", "device": "dev_phone", "seq": 1,
                    "keyHex": hex::encode(keys.c2p), "plain": r#"{"t":"sync"}"#, "body": c2p,
                },
                {
                    "direction": "phoneToComputer", "device": "dev_phone", "seq": 7,
                    "keyHex": hex::encode(keys.p2c), "plain": "olá 🔥", "body": p2c,
                },
            ],
        });
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-vectors/mobile.json");
        let text = serde_json::to_string_pretty(&vectors).expect("json") + "\n";
        if std::env::var_os("BOTLOFT_WRITE_VECTORS").is_some() {
            std::fs::write(&path, &text).expect("write the vectors");
        }
        let on_disk = std::fs::read_to_string(&path).expect("docs/test-vectors/mobile.json");
        assert_eq!(
            on_disk.replace("\r\n", "\n"),
            text,
            "run with BOTLOFT_WRITE_VECTORS=1"
        );
    }
}
