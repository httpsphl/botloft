//! A task that waits for the store holds up no other task: timers fire and
//! connections are served while a slow request has the store.

mod common;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use botloftd::state::Event;
use common::TestDaemon;
use common::bots::two_bots;
use common::stream;

/// How long the store stays held.
const HELD: Duration = Duration::from_millis(1000);

/// Holds the store on a plain thread, like a slow request on the blocking
/// pool, and returns once it is held. The courier wakes on its own timer
/// meanwhile (every 10 ms here) and waits for the store on a runtime
/// thread.
fn hold_store(t: &TestDaemon) -> std::thread::JoinHandle<()> {
    let (locked, is_locked) = std::sync::mpsc::channel();
    let holder = std::thread::spawn({
        let daemon = Arc::clone(&t.daemon);
        move || {
            let _store = daemon.store();
            locked.send(()).expect("locked");
            std::thread::sleep(HELD);
        }
    });
    is_locked.recv().expect("store held");
    // Long enough for the courier to be waiting. A tokio sleep here would
    // itself depend on the timers.
    std::thread::sleep(Duration::from_millis(50));
    holder
}

/// `GET /health` from a plain thread: how long the answer took.
fn health(addr: SocketAddr) -> Duration {
    let asked = Instant::now();
    let mut socket = TcpStream::connect(addr).expect("connect");
    socket
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .expect("ask");
    let mut answer = String::new();
    socket.read_to_string(&mut answer).expect("answer");
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    asked.elapsed()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_daemon_answers_while_the_store_is_held() {
    let c = two_bots().await;
    let addr = c.t.addr;
    let holder = hold_store(&c.t);
    // The runtime thread this test runs on stays free meanwhile.
    let waited = tokio::task::spawn_blocking(move || health(addr))
        .await
        .expect("health");
    holder.join().expect("holder");
    assert!(waited < HELD / 2, "/health waited {waited:?} for the store");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn live_text_goes_out_while_the_store_is_held() {
    let c = two_bots().await;
    let mut events = c.t.daemon.subscribe();
    let holder = hold_store(&c.t);
    let sent = Instant::now();
    // Live text needs no store: it goes out on the 40 ms timer (spec 8.3).
    c.lead_process
        .output(format!("{}\n", stream::delta("Still here.")).as_bytes())
        .await;
    loop {
        if let Ok(Event::ChatDelta(delta)) = events.recv().await {
            assert_eq!(delta.text, "Still here.");
            break;
        }
    }
    let waited = sent.elapsed();
    holder.join().expect("holder");
    assert!(
        waited < HELD / 2,
        "live text waited {waited:?} for the store"
    );
}
