//! Asking a daemon on 127.0.0.1 whether it is up, over plain HTTP.

use std::io::{Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use serde::Deserialize;

const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Health {
    pub status: String,
    pub version: String,
    pub protocol: u32,
}

/// The daemon's `/health`, or `None` if nothing Botloft answers.
pub fn probe(port: u16) -> Option<Health> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, TIMEOUT).ok()?;
    stream.set_read_timeout(Some(TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(TIMEOUT)).ok()?;
    let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).ok()?;
    let response = String::from_utf8_lossy(&response);
    let (head, body) = response.split_once("\r\n\r\n")?;
    if !head.starts_with("HTTP/1.1 200") {
        return None;
    }
    serde_json::from_str::<Health>(body)
        .ok()
        .filter(|health| health.status == "ok")
}

/// Polls until `done` holds for the probe, or `limit` passes.
pub fn wait(port: u16, limit: Duration, done: impl Fn(Option<&Health>) -> bool) -> bool {
    let started = Instant::now();
    loop {
        if done(probe(port).as_ref()) {
            return true;
        }
        if started.elapsed() > limit {
            return false;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
