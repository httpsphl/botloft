//! Tauri shell for the Botloft app. The UI talks to `botloftd` over
//! WebSocket; native commands (spec 15.2) are added here.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running the Botloft app");
}
