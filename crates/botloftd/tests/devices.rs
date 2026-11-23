//! Devices with a token of their own (spec 42.6): the app on another
//! computer connects with it, the owner sees and revokes it, and only the
//! owner's token pairs a new one.

mod common;

use botloft_core::protocol::DevicesPairParams;
use common::{Client, TOKEN, TestDaemon};
use serde_json::{Value, json};

const NOT_AUTHENTICATED: i64 = -32001;
const VALIDATION: i64 = -32004;

/// A device's public SSH key, in the form `ssh-keygen -t ed25519` writes.
const SSH_KEY: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB0fX3dnSrK6yp6CmM8kU4DhkBiQ0jFv1UuSGBNPmSdw";

fn laptop_params() -> DevicesPairParams {
    DevicesPairParams {
        name: "Laptop".to_owned(),
        ssh_key: None,
    }
}

/// Pairs a device called `name` as the owner; its id and token.
async fn pair(owner: &mut Client, name: &str) -> (Value, String) {
    let paired = owner
        .call("devices.pair", json!({ "name": name }))
        .await
        .expect("pair");
    let token = paired["token"].as_str().expect("token").to_owned();
    (paired["device"]["id"].clone(), token)
}

#[tokio::test]
async fn a_paired_device_connects_with_its_own_token() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let (device, token) = pair(&mut owner, "Laptop").await;
    assert!(device.as_str().expect("id").starts_with("dev_"));
    assert_ne!(token, TOKEN);

    let mut laptop = t.client().await;
    let hello = laptop.hello(&token).await.expect("hello");
    assert_eq!(hello["deviceId"], device);
    laptop.call("crews.list", Value::Null).await.expect("works");

    let owner_hello = t.client().await.hello(TOKEN).await.expect("owner");
    assert!(
        owner_hello.get("deviceId").is_none(),
        "the owner is no device"
    );

    let list = owner.call("devices.list", Value::Null).await.expect("list");
    let devices = list["devices"].as_array().expect("devices");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0]["name"], "Laptop");
    assert!(devices[0]["lastSeenAt"].is_i64(), "seen when it connected");
}

#[tokio::test]
async fn the_owner_hears_of_a_new_device() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let mut other = t.session().await;
    pair(&mut owner, "Laptop").await;
    let changed = other.notification("devices.changed").await;
    assert_eq!(changed["devices"][0]["name"], "Laptop");
    assert!(
        changed.to_string().find("token").is_none(),
        "no token goes out"
    );
}

#[tokio::test]
async fn only_the_owner_pairs_a_device() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let (_, token) = pair(&mut owner, "Laptop").await;
    let mut laptop = t.client().await;
    laptop.hello(&token).await.expect("hello");
    let refused = laptop
        .call("devices.pair", json!({ "name": "Another" }))
        .await
        .expect_err("a device cannot pair another");
    assert_eq!(refused.code, VALIDATION);
    assert_eq!(refused.reason, "pair_where_botloft_runs");
    let list = owner.call("devices.list", Value::Null).await.expect("list");
    assert_eq!(list["devices"].as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn a_revoked_device_is_closed_and_its_token_refused() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let (device, token) = pair(&mut owner, "Laptop").await;
    let (_, kept_token) = pair(&mut owner, "Desk").await;
    let mut laptop = t.client().await;
    laptop.hello(&token).await.expect("hello");
    let mut desk = t.client().await;
    desk.hello(&kept_token).await.expect("hello");

    let left = owner
        .call("devices.revoke", json!({ "deviceId": device }))
        .await
        .expect("revoke");
    assert_eq!(left["devices"].as_array().map(Vec::len), Some(1));
    assert!(
        laptop.closed().await,
        "the revoked device's connection closes"
    );
    desk.call("crews.list", Value::Null)
        .await
        .expect("another device stays");

    let refused = t
        .client()
        .await
        .hello(&token)
        .await
        .expect_err("the token stopped working");
    assert_eq!(refused.code, NOT_AUTHENTICATED);
    let again = owner
        .call("devices.revoke", json!({ "deviceId": device }))
        .await
        .expect_err("already gone");
    assert_eq!(again.code, -32002);
}

#[tokio::test]
async fn a_device_may_revoke_itself_and_hears_the_answer() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let (device, token) = pair(&mut owner, "Laptop").await;
    let mut laptop = t.client().await;
    laptop.hello(&token).await.expect("hello");
    let answer = laptop
        .call("devices.revoke", json!({ "deviceId": device }))
        .await
        .expect("the answer comes before the close");
    assert_eq!(answer["devices"], json!([]));
    assert!(laptop.closed().await);
}

#[tokio::test]
async fn the_command_line_pairs_with_the_owners_token() {
    let t = TestDaemon::start().await;
    let paired = botloftd::pair::pair(t.addr.port(), TOKEN, &laptop_params())
        .await
        .expect("pair");
    assert_eq!(paired.device.name, "Laptop");
    let mut laptop = t.client().await;
    laptop
        .hello(&paired.token)
        .await
        .expect("the key's token works");

    let wrong = botloftd::pair::pair(t.addr.port(), "not-the-owner", &laptop_params())
        .await
        .expect_err("a wrong token pairs nothing");
    assert!(wrong.to_string().contains("refused"), "{wrong}");
}

#[tokio::test]
async fn a_device_brings_its_ssh_key_and_takes_it_along() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let paired = owner
        .call(
            "devices.pair",
            json!({ "name": "Laptop", "sshKey": format!("{SSH_KEY} botloft") }),
        )
        .await
        .expect("pair");
    let keys = || t.daemon.store().device_ssh_keys().expect("keys");
    assert_eq!(keys(), [SSH_KEY], "kept without its comment");

    owner
        .call(
            "devices.revoke",
            json!({ "deviceId": paired["device"]["id"] }),
        )
        .await
        .expect("revoke");
    assert!(keys().is_empty(), "sshd no longer lets it in");
}

#[tokio::test]
async fn a_key_that_is_not_ed25519_pairs_nothing() {
    let t = TestDaemon::start().await;
    let mut owner = t.session().await;
    let err = owner
        .call(
            "devices.pair",
            json!({ "name": "Laptop", "sshKey": format!("command=\"sh\" {SSH_KEY}") }),
        )
        .await
        .expect_err("refused");
    assert_eq!(err.code, VALIDATION);
    let list = owner.call("devices.list", Value::Null).await.expect("list");
    assert_eq!(list["devices"], json!([]));
}
