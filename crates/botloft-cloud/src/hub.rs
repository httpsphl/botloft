//! Who is connected to the relay right now (spec 28.4): one socket per
//! device, found by its id. Nothing here is stored; it is the live pipes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

/// What a socket's writer is asked to do.
pub enum Out {
    Text(String),
    Close,
}

struct Conn {
    id: u64,
    tx: mpsc::Sender<Out>,
}

#[derive(Default)]
struct Inner {
    next: u64,
    conns: HashMap<String, Conn>,
}

#[derive(Clone, Default)]
pub struct Hub(Arc<Mutex<Inner>>);

impl Hub {
    fn inner(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Registers the socket of `device`; a second socket of the same device
    /// replaces the first, which is told to close. The number goes back to
    /// `detach`.
    pub fn attach(&self, device: &str, tx: mpsc::Sender<Out>) -> u64 {
        let mut inner = self.inner();
        inner.next += 1;
        let id = inner.next;
        if let Some(old) = inner.conns.insert(device.to_owned(), Conn { id, tx }) {
            let _ = old.tx.try_send(Out::Close);
        }
        id
    }

    /// Drops the registration, unless a newer socket took the place.
    pub fn detach(&self, device: &str, id: u64) {
        let mut inner = self.inner();
        if inner.conns.get(device).is_some_and(|conn| conn.id == id) {
            inner.conns.remove(device);
        }
    }

    pub fn online(&self, device: &str) -> bool {
        self.inner().conns.contains_key(device)
    }

    /// Hands `text` to the device's socket. False when it is not connected or
    /// cannot keep up (a slow reader loses the frame, the queue keeps what
    /// matters).
    pub fn send(&self, device: &str, text: String) -> bool {
        let inner = self.inner();
        inner
            .conns
            .get(device)
            .is_some_and(|conn| conn.tx.try_send(Out::Text(text)).is_ok())
    }

    /// Closes the device's socket: its token is gone.
    pub fn kick(&self, device: &str) {
        if let Some(conn) = self.inner().conns.remove(device) {
            let _ = conn.tx.try_send(Out::Close);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_newer_socket_replaces_the_older_and_the_older_cannot_detach_it() {
        let hub = Hub::default();
        let (first, mut first_rx) = mpsc::channel(4);
        let (second, mut second_rx) = mpsc::channel(4);
        let a = hub.attach("dev", first);
        let b = hub.attach("dev", second);
        assert!(matches!(first_rx.try_recv(), Ok(Out::Close)));
        hub.detach("dev", a);
        assert!(hub.online("dev"));
        assert!(hub.send("dev", "hi".into()));
        assert!(matches!(second_rx.try_recv(), Ok(Out::Text(text)) if text == "hi"));
        hub.detach("dev", b);
        assert!(!hub.online("dev"));
        assert!(!hub.send("dev", "lost".into()));
    }

    #[test]
    fn a_full_socket_loses_the_frame_and_a_kick_closes() {
        let hub = Hub::default();
        let (tx, mut rx) = mpsc::channel(1);
        hub.attach("dev", tx);
        assert!(hub.send("dev", "one".into()));
        assert!(!hub.send("dev", "two".into()));
        hub.kick("dev");
        assert!(!hub.online("dev"));
        assert!(matches!(rx.try_recv(), Ok(Out::Text(_))));
    }
}
