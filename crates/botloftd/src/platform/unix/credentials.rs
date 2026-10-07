//! No credential store on Linux and macOS yet (spec 27.10): the automatic
//! backup needs one to keep its passphrase, so it stays off there.

use std::io;

pub fn secret_available() -> bool {
    false
}

fn unsupported() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "there is no credential store on this system",
    )
}

pub fn secret_save(_target: &str, _secret: &str) -> io::Result<()> {
    Err(unsupported())
}

pub fn secret_load(_target: &str) -> io::Result<Option<String>> {
    Ok(None)
}

pub fn secret_delete(_target: &str) -> io::Result<()> {
    Ok(())
}
