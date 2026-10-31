//! The app's own commands under Tauri's ACL (spec 31.3): `build.rs` lists
//! every command `lib.rs` registers, so each gets an `allow-` permission and
//! no window may call it unless its capability grants it. The main window
//! is granted all of them; the island only the few it needs, and nothing
//! that opens files, links or folders.

use serde_json::Value;

/// The quoted names in `build.rs`'s `COMMANDS`.
fn listed() -> Vec<String> {
    let build = include_str!("../build.rs");
    let list = build
        .split_once("const COMMANDS")
        .and_then(|(_, rest)| rest.split_once("= ["))
        .and_then(|(_, rest)| rest.split_once("];"))
        .map(|(list, _)| list)
        .expect("COMMANDS in build.rs");
    names(list)
        .map(|name| name.trim_matches('"').to_owned())
        .collect()
}

/// The functions in `lib.rs`'s `generate_handler!`.
fn registered() -> Vec<String> {
    let lib = include_str!("../src/lib.rs");
    let list = lib
        .split_once("generate_handler![")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .expect("generate_handler! in lib.rs");
    names(list).map(str::to_owned).collect()
}

fn names(list: &str) -> impl Iterator<Item = &str> {
    list.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// The app's own permissions a capability grants, in its order.
fn granted(capability: &str) -> Vec<String> {
    let json: Value = serde_json::from_str(capability).expect("capability JSON");
    json["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .filter_map(Value::as_str)
        .filter(|permission| permission.starts_with("allow-"))
        .map(str::to_owned)
        .collect()
}

fn allow(command: &str) -> String {
    format!("allow-{}", command.replace('_', "-"))
}

#[test]
fn every_registered_command_is_listed_for_the_acl() {
    let mut listed = listed();
    let mut registered = registered();
    listed.sort();
    registered.sort();
    assert_eq!(listed, registered);
}

#[test]
fn the_main_window_may_call_every_command() {
    let mut main = granted(include_str!("../capabilities/default.json"));
    let mut all: Vec<String> = listed().iter().map(|command| allow(command)).collect();
    main.sort();
    all.sort();
    assert_eq!(main, all);
}

#[test]
fn the_island_may_call_only_its_own_few() {
    let island = granted(include_str!("../capabilities/island.json"));
    let own = [
        "daemon_status",
        "read_owner_token",
        "quiet_now",
        "main_in_front",
    ]
    .map(allow);
    assert_eq!(island, own);
}
