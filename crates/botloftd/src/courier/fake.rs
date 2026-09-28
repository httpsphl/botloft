//! An inbox that records what the courier posts instead of writing to a
//! pipe, for tests. It can fail on demand or hold writes until released.

use std::io;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use bytes::Bytes;
use futures_util::future::BoxFuture;
use serde_json::Value;
use tokio::sync::{Notify, watch};

use super::inbox::InboxWriter;

/// One message that reached an inbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub address: String,
    /// Token from the auth line.
    pub token: String,
    /// The envelope, as the bot reads it.
    pub text: String,
}

#[derive(Clone)]
pub struct FakeInbox {
    state: Arc<Mutex<State>>,
    posted: Arc<Notify>,
    held: watch::Sender<bool>,
}

#[derive(Default)]
struct State {
    posts: Vec<Post>,
    failures: usize,
    attempts: usize,
}

impl Default for FakeInbox {
    fn default() -> Self {
        Self {
            state: Arc::default(),
            posted: Arc::default(),
            held: watch::Sender::new(false),
        }
    }
}

impl FakeInbox {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn posts(&self) -> Vec<Post> {
        self.lock().posts.clone()
    }

    /// Writes started so far, failed ones included.
    pub fn attempts(&self) -> usize {
        self.lock().attempts
    }

    /// The `n`-th post (1-based), waiting up to 5 s for it.
    pub async fn post(&self, n: usize) -> Post {
        let wait = async {
            loop {
                let notified = self.posted.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if let Some(post) = self.lock().posts.get(n - 1) {
                    return post.clone();
                }
                notified.await;
            }
        };
        tokio::time::timeout(Duration::from_secs(5), wait)
            .await
            .unwrap_or_else(|_| panic!("post {n} never arrived"))
    }

    /// Makes the next `n` writes fail, as a closed pipe would.
    pub fn fail_next(&self, n: usize) {
        self.lock().failures += n;
    }

    /// Writes wait until [`FakeInbox::release`].
    pub fn hold(&self) {
        self.held.send_replace(true);
    }

    pub fn release(&self) {
        self.held.send_replace(false);
    }
}

impl InboxWriter for FakeInbox {
    fn write<'a>(&'a self, address: &'a str, payload: Bytes) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move {
            self.lock().attempts += 1;
            let mut held = self.held.subscribe();
            let _ = held.wait_for(|held| !held).await;
            {
                let mut state = self.lock();
                if state.failures > 0 {
                    state.failures -= 1;
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "fake failure"));
                }
                let post = parse(address, &payload)?;
                state.posts.push(post);
            }
            self.posted.notify_waiters();
            Ok(())
        })
    }
}

/// Reads the two lines back, checking their shape like Claude Code would.
fn parse(address: &str, payload: &[u8]) -> io::Result<Post> {
    let invalid = |what: &str| io::Error::new(io::ErrorKind::InvalidData, what.to_owned());
    let text = std::str::from_utf8(payload).map_err(|_| invalid("not UTF-8"))?;
    let mut lines = text.split_terminator('\n');
    let mut next = || -> io::Result<Value> {
        let line = lines.next().ok_or_else(|| invalid("missing line"))?;
        serde_json::from_str(line).map_err(|_| invalid("not a JSON line"))
    };
    let auth = next()?;
    let message = next()?;
    let field = |value: &Value, name: &str| value[name].as_str().map(str::to_owned);
    if field(&auth, "type").as_deref() != Some("auth")
        || field(&message, "type").as_deref() != Some("user")
    {
        return Err(invalid("unexpected line types"));
    }
    Ok(Post {
        address: address.to_owned(),
        token: field(&auth, "token").ok_or_else(|| invalid("no token"))?,
        text: field(&message["message"], "content").ok_or_else(|| invalid("no content"))?,
    })
}
