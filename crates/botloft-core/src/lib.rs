//! Domain types, IDs, message envelope rendering and protocol types shared by
//! the daemon and the app.

pub mod avatar;
pub mod chat;
pub mod command;
pub mod envelope;
pub mod ids;
pub mod protocol;
pub mod slug;
pub mod validate;

pub use protocol::PROTOCOL_VERSION;

/// Current time in milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
}
