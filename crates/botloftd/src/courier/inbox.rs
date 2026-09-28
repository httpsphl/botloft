//! How a rendered envelope reaches a bot: the two lines Claude Code reads
//! from an inbox connection (spec 9.2), and the writer that sends them.

use std::io;

use bytes::Bytes;
use futures_util::future::BoxFuture;
use serde_json::json;

use crate::platform;

/// Writes one payload to an inbox and closes the connection.
pub trait InboxWriter: Send + Sync + 'static {
    fn write<'a>(&'a self, address: &'a str, payload: Bytes) -> BoxFuture<'a, io::Result<()>>;
}

/// The real inbox: a named pipe on Windows, a Unix socket elsewhere.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipeInbox;

impl InboxWriter for PipeInbox {
    fn write<'a>(&'a self, address: &'a str, payload: Bytes) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { platform::write_inbox(address, &payload).await })
    }
}

/// The auth line, then the message line, each ending in `\n`. Holds the
/// messaging token and the body: never log it.
pub fn payload(token: &str, envelope: &str) -> Bytes {
    let auth = json!({ "type": "auth", "token": token });
    let message = json!({
        "type": "user",
        "message": { "role": "user", "content": envelope },
    });
    Bytes::from(format!("{auth}\n{message}\n"))
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn the_payload_is_two_json_lines_auth_first() {
        let payload = payload("tok", "[botloft] from the owner\n\nOlá, \"bot\"");
        let text = std::str::from_utf8(&payload).expect("utf8");
        assert!(text.ends_with('\n'));
        let lines: Vec<Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).expect("json line"))
            .collect();
        assert_eq!(
            lines.len(),
            2,
            "one line each; newlines in the body are escaped"
        );
        assert_eq!(lines[0], json!({ "type": "auth", "token": "tok" }));
        assert_eq!(lines[1]["type"], "user");
        assert_eq!(lines[1]["message"]["role"], "user");
        assert_eq!(
            lines[1]["message"]["content"],
            "[botloft] from the owner\n\nOlá, \"bot\""
        );
    }
}
