//! Keeps the computer awake while any bot is `busy`, when `keep_awake` is
//! on (spec 14). A bot waiting for the owner or idle lets it sleep.

use tokio::sync::watch;
use tracing::{debug, warn};

use crate::platform::KeepAwake;

/// Follows the supervisor's count of busy bots until the daemon stops.
pub async fn run(busy: watch::Receiver<usize>) {
    let mut request = match KeepAwake::new() {
        Ok(request) => request,
        Err(err) => {
            warn!("cannot keep the computer awake: {err}");
            return;
        }
    };
    follow(busy, |on| {
        debug!(on, "keep awake");
        if let Err(err) = request.set(on) {
            warn!("cannot change the keep-awake request: {err}");
        }
    })
    .await;
}

/// Calls `hold` with `true` when the first bot starts working and with
/// `false` when the last one stops.
async fn follow(mut busy: watch::Receiver<usize>, mut hold: impl FnMut(bool)) {
    let mut held = false;
    loop {
        let working = *busy.borrow_and_update() > 0;
        if working != held {
            hold(working);
            held = working;
        }
        if busy.changed().await.is_err() {
            break;
        }
    }
    if held {
        hold(false);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[tokio::test]
    async fn holds_only_while_some_bot_is_busy() {
        let (count, busy) = watch::channel(0usize);
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&calls);
        let task = tokio::spawn(follow(busy, move |on| {
            seen.lock().expect("calls").push(on);
        }));
        for n in [1, 2, 1, 0, 0, 3] {
            count.send_replace(n);
            tokio::task::yield_now().await;
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        drop(count);
        task.await.expect("follow ends with the daemon");
        assert_eq!(*calls.lock().expect("calls"), [true, false, true, false]);
    }

    #[test]
    fn a_keep_awake_request_can_be_set_and_cleared() {
        let mut request = KeepAwake::new().expect("request");
        request.set(true).expect("set");
        request.set(true).expect("set twice");
        request.set(false).expect("clear");
    }
}
