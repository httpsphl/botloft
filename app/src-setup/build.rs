//! Puts the NSIS installer inside the setup (spec 15.7). Its path comes in
//! `BOTLOFT_SETUP_PAYLOAD`; without it, as in dev builds and clippy, the
//! payload is empty and the setup only rehearses.

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=windows/setup.manifest");
    println!("cargo:rerun-if-env-changed=BOTLOFT_SETUP_PAYLOAD");
    let out =
        PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR")).join("payload.exe");
    match env::var_os("BOTLOFT_SETUP_PAYLOAD") {
        Some(path) => {
            println!("cargo:rerun-if-changed={}", PathBuf::from(&path).display());
            fs::copy(&path, &out).expect("BOTLOFT_SETUP_PAYLOAD must point to the NSIS installer");
        }
        None => fs::write(&out, []).expect("cannot write the empty payload"),
    }

    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows/setup.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri-build");
}
