//! Files dropped on a window reach its page (spec 15.1): with Tauri's own
//! drag-and-drop on, Windows hands the drop to Tauri and the page never
//! sees it, so dropping a file on the chat or the island attached nothing.

use serde_json::Value;

#[test]
fn every_window_lets_its_page_take_dropped_files() {
    let config: Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("tauri.conf.json");
    let windows = config["app"]["windows"].as_array().expect("windows");
    assert!(!windows.is_empty());
    for window in windows {
        assert_eq!(
            window["dragDropEnabled"],
            Value::Bool(false),
            "window {} must leave dropped files to its page",
            window["label"]
        );
    }
}
