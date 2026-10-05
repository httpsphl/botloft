//! The sealed form of a backup (spec 14.2): what is written to disk is
//! never readable without the owner's passphrase.
//!
//! Layout: `MAGIC`, a version byte, the Argon2id salt and costs, the stream
//! nonce, then the archive in chunks of `CHUNK` bytes, each encrypted with
//! XChaCha20-Poly1305 in the STREAM construction (big-endian 32-bit
//! counter, last-chunk flag), so a cut or reordered file fails to open.

use std::io::{self, Read, Write};

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::stream::{DecryptorBE32, EncryptorBE32};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};

const MAGIC: &[u8; 10] = b"BOTLOFTBAK";
const VERSION: u8 = 1;
const SALT_LEN: usize = 16;
/// XChaCha20's 24-byte nonce less STREAM's 5 bytes of counter and flag.
const NONCE_LEN: usize = 19;
const TAG_LEN: usize = 16;
/// Plaintext per chunk.
const CHUNK: usize = 64 * 1024;
/// Argon2id costs: 64 MiB, three passes, one lane.
const MEMORY_KIB: u32 = if cfg!(test) { 1024 } else { 64 * 1024 };
const PASSES: u32 = 3;
const LANES: u32 = 1;
/// The shortest passphrase accepted, in characters.
pub const PASSPHRASE_MIN: usize = 8;

#[derive(Debug, thiserror::Error)]
pub enum SealError {
    #[error("this is not a Botloft backup")]
    NotABackup,
    #[error("this backup was made by a newer Botloft; update Botloft to open it")]
    Newer,
    #[error("the passphrase is wrong, or the file is damaged")]
    WrongPassphrase,
    #[error("the passphrase must have at least {PASSPHRASE_MIN} characters")]
    ShortPassphrase,
    #[error(transparent)]
    Io(#[from] io::Error),
}

struct Header {
    salt: [u8; SALT_LEN],
    memory: u32,
    passes: u32,
    lanes: u32,
    nonce: [u8; NONCE_LEN],
}

fn key(passphrase: &str, header: &Header) -> Result<[u8; 32], SealError> {
    let params = Params::new(header.memory, header.passes, header.lanes, Some(32))
        .map_err(|_| SealError::NotABackup)?;
    let mut key = [0u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(passphrase.as_bytes(), &header.salt, &mut key)
        .map_err(|_| SealError::NotABackup)?;
    Ok(key)
}

fn random<const N: usize>() -> io::Result<[u8; N]> {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).map_err(|err| io::Error::other(err.to_string()))?;
    Ok(bytes)
}

/// Reads `plain` to its end and writes it sealed with `passphrase`.
pub fn seal(
    plain: &mut impl Read,
    out: &mut impl Write,
    passphrase: &str,
) -> Result<(), SealError> {
    if passphrase.chars().count() < PASSPHRASE_MIN {
        return Err(SealError::ShortPassphrase);
    }
    let header = Header {
        salt: random()?,
        memory: MEMORY_KIB,
        passes: PASSES,
        lanes: LANES,
        nonce: random()?,
    };
    out.write_all(MAGIC)?;
    out.write_all(&[VERSION])?;
    out.write_all(&header.salt)?;
    for cost in [header.memory, header.passes, header.lanes] {
        out.write_all(&cost.to_le_bytes())?;
    }
    out.write_all(&header.nonce)?;
    let key = key(passphrase, &header)?;
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream = EncryptorBE32::from_aead(cipher, header.nonce.as_ref().into());
    let mut chunk = read_chunk(plain, CHUNK)?;
    loop {
        let next = read_chunk(plain, CHUNK)?;
        if next.is_empty() {
            let last = stream
                .encrypt_last(chunk.as_slice())
                .map_err(|_| io::Error::other("could not encrypt"))?;
            out.write_all(&last)?;
            return Ok(out.flush()?);
        }
        let sealed = stream
            .encrypt_next(chunk.as_slice())
            .map_err(|_| io::Error::other("could not encrypt"))?;
        out.write_all(&sealed)?;
        chunk = next;
    }
}

/// Reads a sealed backup and writes what it holds, or fails without
/// writing a chunk that does not open.
pub fn open(
    sealed: &mut impl Read,
    out: &mut impl Write,
    passphrase: &str,
) -> Result<(), SealError> {
    let mut magic = [0u8; MAGIC.len()];
    sealed
        .read_exact(&mut magic)
        .map_err(|_| SealError::NotABackup)?;
    if &magic != MAGIC {
        return Err(SealError::NotABackup);
    }
    let mut version = [0u8; 1];
    sealed.read_exact(&mut version)?;
    if version[0] != VERSION {
        return Err(SealError::Newer);
    }
    let mut salt = [0u8; SALT_LEN];
    sealed.read_exact(&mut salt)?;
    let mut costs = [0u32; 3];
    for cost in &mut costs {
        let mut bytes = [0u8; 4];
        sealed.read_exact(&mut bytes)?;
        *cost = u32::from_le_bytes(bytes);
    }
    let mut nonce = [0u8; NONCE_LEN];
    sealed.read_exact(&mut nonce)?;
    let header = Header {
        salt,
        memory: costs[0],
        passes: costs[1],
        lanes: costs[2],
        nonce,
    };
    let key = key(passphrase, &header)?;
    let cipher = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream = DecryptorBE32::from_aead(cipher, header.nonce.as_ref().into());
    let mut chunk = read_chunk(sealed, CHUNK + TAG_LEN)?;
    loop {
        let next = read_chunk(sealed, CHUNK + TAG_LEN)?;
        if next.is_empty() {
            let last = stream
                .decrypt_last(chunk.as_slice())
                .map_err(|_| SealError::WrongPassphrase)?;
            out.write_all(&last)?;
            return Ok(out.flush()?);
        }
        let plain = stream
            .decrypt_next(chunk.as_slice())
            .map_err(|_| SealError::WrongPassphrase)?;
        out.write_all(&plain)?;
        chunk = next;
    }
}

/// Up to `size` bytes; fewer only at the end.
fn read_chunk(from: &mut impl Read, size: usize) -> io::Result<Vec<u8>> {
    let mut chunk = vec![0u8; size];
    let mut filled = 0;
    while filled < size {
        match from.read(&mut chunk[filled..])? {
            0 => break,
            read => filled += read,
        }
    }
    chunk.truncate(filled);
    Ok(chunk)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(size: usize) {
        let plain: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        let mut sealed = Vec::new();
        seal(&mut plain.as_slice(), &mut sealed, "correct horse").expect("seal");
        assert!(
            !sealed
                .windows(16)
                .any(|w| plain.len() >= 16 && w == &plain[..16])
        );
        let mut opened = Vec::new();
        open(&mut sealed.as_slice(), &mut opened, "correct horse").expect("open");
        assert_eq!(opened, plain);
    }

    #[test]
    fn what_is_sealed_opens_with_the_passphrase_at_any_size() {
        for size in [0, 1, CHUNK - 1, CHUNK, CHUNK + 1, 3 * CHUNK + 7] {
            round_trip(size);
        }
    }

    #[test]
    fn a_wrong_passphrase_a_cut_file_and_another_file_do_not_open() {
        let plain = vec![7u8; 2 * CHUNK + 10];
        let mut sealed = Vec::new();
        seal(&mut plain.as_slice(), &mut sealed, "correct horse").expect("seal");
        let mut out = Vec::new();
        assert!(matches!(
            open(&mut sealed.as_slice(), &mut out, "wrong horse"),
            Err(SealError::WrongPassphrase)
        ));
        // Without its last chunk, the file stops at a chunk not marked last.
        let cut = &sealed[..sealed.len() - 20];
        assert!(open(&mut &cut[..], &mut Vec::new(), "correct horse").is_err());
        assert!(matches!(
            open(
                &mut &b"PK\x03\x04 a zip"[..],
                &mut Vec::new(),
                "correct horse"
            ),
            Err(SealError::NotABackup)
        ));
        assert!(matches!(
            seal(&mut &b"x"[..], &mut Vec::new(), "short"),
            Err(SealError::ShortPassphrase)
        ));
    }
}
