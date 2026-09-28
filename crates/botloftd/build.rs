//! Embeds `botloftd.manifest` in the Windows binary (spec 14). It goes in
//! as a resource: the linker's own manifest tool predates
//! `consoleAllocationPolicy` and warns about it on every build.

fn main() {
    println!("cargo:rerun-if-changed=botloftd.rc");
    println!("cargo:rerun-if-changed=botloftd.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Err(err) =
            embed_resource::compile("botloftd.rc", embed_resource::NONE).manifest_required()
    {
        panic!("cannot embed the manifest: {err}");
    }
}
