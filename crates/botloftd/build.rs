//! Embeds the daemon's Windows resources (spec 14): `botloftd.manifest` and
//! the version information. The manifest goes in as a resource because the
//! linker's own manifest tool predates `consoleAllocationPolicy` and warns
//! about it on every build.

use std::path::PathBuf;
use std::{env, fs};

/// `1.2.3` (with or without a tag) as the four numbers Windows wants.
fn version_numbers(version: &str) -> String {
    let core = version.split(['-', '+']).next().unwrap_or_default();
    let mut parts: Vec<&str> = core.split('.').collect();
    parts.resize(4, "0");
    parts.join(",")
}

fn main() {
    println!("cargo:rerun-if-changed=botloftd.rc");
    println!("cargo:rerun-if-changed=botloftd.manifest");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let here = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets the crate dir"));
    let version = env::var("CARGO_PKG_VERSION").expect("cargo sets the version");
    // Forward slashes: a backslash starts an escape in a resource script.
    let manifest = here
        .join("botloftd.manifest")
        .display()
        .to_string()
        .replace('\\', "/");
    let script = fs::read_to_string(here.join("botloftd.rc"))
        .expect("cannot read botloftd.rc")
        .replace("@MANIFEST@", &manifest)
        .replace("@VERSION_NUMBERS@", &version_numbers(&version))
        .replace("@VERSION@", &version);
    let out =
        PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR")).join("botloftd.rc");
    fs::write(&out, script).expect("cannot write the resource script");
    if let Err(err) = embed_resource::compile(&out, embed_resource::NONE).manifest_required() {
        panic!("cannot embed the resources: {err}");
    }
}
