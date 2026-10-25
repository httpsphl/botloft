/// The app's own commands (spec 15.2), as `lib.rs` registers them.
const COMMANDS: [&str; 13] = [
    "daemon_status",
    "daemon_install",
    "daemon_restart",
    "daemon_stop",
    "claude_sign_in",
    "read_owner_token",
    "open_path",
    "open_file",
    "reveal_file",
    "save_file_as",
    "open_url",
    "open_at_sign_in",
    "launched_at_sign_in",
];

fn main() {
    println!("cargo:rerun-if-changed=windows/app.manifest");
    let windows =
        tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows/app.manifest"));
    // Every command must be allowed by a window's capability: the island gets
    // only the two it needs (spec 31.3), the main window all of them.
    let commands = tauri_build::AppManifest::new().commands(&COMMANDS);
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .windows_attributes(windows)
            .app_manifest(commands),
    )
    .expect("failed to run tauri-build");
}
