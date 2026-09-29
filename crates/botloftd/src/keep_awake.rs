//! Keeps the computer awake while any bot is `busy`, when `keep_awake` is
//! on (spec 14). A bot waiting for the owner or idle lets it sleep, and so
//! does the owner turning it off in Settings.

use tokio::sync::watch;
use tracing::{debug, warn};

use crate::platform::KeepAwake;

/// Follows the supervisor's count of busy bots and the owner's setting
/// until the daemon stops.
pub async fn run(busy: watch::Receiver<usize>, enabled: watch::Receiver<bool>) {
    let mut request = match KeepAwake::new() {
        Ok(request) => request,
        Err(err) => {
            warn!("cannot keep the computer awake: {err}");
            return;
        }
    };
    follow(busy, enabled, |on| {
        debug!(on, "keep awake");
        if let Err(err) = request.set(on) {
            warn!("cannot change the keep-awake request: {err}");
        }
    })
    .await;
}

/// Calls `hold` with `true` when the first bot starts working and with
/// `false` when the last one stops, while `enabled`.
async fn follow(
    mut busy: watch::Receiver<usize>,
    mut enabled: watch::Receiver<bool>,
    mut hold: impl FnMut(bool),
) {
    let mut held = false;
    loop {
        let wanted = *enabled.borrow_and_update() && *busy.borrow_and_update() > 0;
        if wanted != held {
            hold(wanted);
            held = wanted;
        }
        let gone = tokio::select! {
            changed = busy.changed() => changed.is_err(),
            changed = enabled.changed() => changed.is_err(),
        };
        if gone {
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

    type Calls = Arc<Mutex<Vec<bool>>>;

    fn start(
        busy: watch::Receiver<usize>,
        enabled: watch::Receiver<bool>,
    ) -> (tokio::task::JoinHandle<()>, Calls) {
        let calls = Calls::default();
        let seen = Arc::clone(&calls);
        let task = tokio::spawn(follow(busy, enabled, move |on| {
            seen.lock().expect("calls").push(on);
        }));
        (task, calls)
    }

    async fn settle() {
        tokio::task::yield_now().await;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    #[tokio::test]
    async fn holds_only_while_some_bot_is_busy() {
        let (count, busy) = watch::channel(0usize);
        let (_setting, enabled) = watch::channel(true);
        let (task, calls) = start(busy, enabled);
        for n in [1, 2, 1, 0, 0, 3] {
            count.send_replace(n);
            settle().await;
        }
        drop(count);
        task.await.expect("follow ends with the daemon");
        assert_eq!(*calls.lock().expect("calls"), [true, false, true, false]);
    }

    #[tokio::test]
    async fn the_owner_can_turn_it_off_and_on_while_bots_work() {
        let (count, busy) = watch::channel(2usize);
        let (setting, enabled) = watch::channel(false);
        let (task, calls) = start(busy, enabled);
        settle().await;
        assert!(calls.lock().expect("calls").is_empty(), "off: never held");
        setting.send_replace(true);
        settle().await;
        setting.send_replace(false);
        settle().await;
        count.send_replace(0);
        setting.send_replace(true);
        settle().await;
        drop(setting);
        task.await.expect("follow ends with the daemon");
        assert_eq!(*calls.lock().expect("calls"), [true, false]);
    }

    #[test]
    fn a_keep_awake_request_can_be_set_and_cleared() {
        let mut request = KeepAwake::new().expect("request");
        request.set(true).expect("set");
        request.set(true).expect("set twice");
        request.set(false).expect("clear");
    }
}
