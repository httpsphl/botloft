//! `botloftd hook <event>`, which Claude Code runs for every hook (spec 7.6).
//!
//! It always exits 0 and never writes to stdout: a SessionStart hook's
//! stdout becomes part of Claude's context. Failures go to
//! `<BOTLOFT_HOME>\logs\hook.log`, without the token.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;

use super::{EVENTS, HookRequest};

const TIMEOUT: Duration = Duration::from_secs(3);
const MAX_STDIN: u64 = 1 << 20;

/// Entry point of the subcommand. Never fails.
pub fn run(event: &str) {
    if let Err(err) = forward(event, &mut io::stdin().take(MAX_STDIN)) {
        log_failure(event, &err);
    }
}

fn forward(event: &str, stdin: &mut impl Read) -> Result<(), String> {
    if !EVENTS.iter().any(|(_, arg)| *arg == event) {
        return Err(format!("unknown hook event `{event}`"));
    }
    let token = var("BOTLOFT_BOT_TOKEN")?;
    let port: u16 = var("BOTLOFT_PORT")?
        .parse()
        .map_err(|_| "BOTLOFT_PORT is not a port number".to_owned())?;

    let mut input = Vec::new();
    stdin
        .read_to_end(&mut input)
        .map_err(|err| format!("cannot read stdin: {err}"))?;
    let mut request = HookRequest {
        payload: serde_json::from_slice(&input).unwrap_or(Value::Null),
        ..HookRequest::default()
    };
    if event == "session-start" {
        request.messaging_socket = std::env::var("CLAUDE_CODE_MESSAGING_SOCKET").ok();
        request.messaging_token = std::env::var("CLAUDE_CODE_MESSAGING_TOKEN").ok();
    }
    let body = serde_json::to_vec(&request).map_err(|err| err.to_string())?;
    post(port, &format!("/hooks/{event}"), &token, &body)
}

fn var(name: &str) -> Result<String, String> {
    std::env::var(name)
        .map_err(|_| format!("{name} is not set; is this running inside a Botloft bot?"))
}

/// A minimal HTTP/1.1 POST to the daemon on 127.0.0.1.
pub fn post(port: u16, path: &str, token: &str, body: &[u8]) -> Result<(), String> {
    let fail = |step: &str, err: io::Error| format!("{step} 127.0.0.1:{port} failed: {err}");
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream =
        TcpStream::connect_timeout(&addr, TIMEOUT).map_err(|e| fail("connecting to", e))?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| fail("configuring", e))?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(|e| fail("configuring", e))?;
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|()| stream.write_all(body))
        .map_err(|e| fail("writing to", e))?;
    let mut status = String::new();
    BufReader::new(stream)
        .read_line(&mut status)
        .map_err(|e| fail("reading from", e))?;
    match status.split_whitespace().nth(1) {
        Some(code) if code.starts_with('2') => Ok(()),
        Some(code) => Err(format!("the daemon answered {code} to {path}")),
        None => Err(format!("the daemon closed the connection on {path}")),
    }
}

fn log_failure(event: &str, err: &str) {
    let Some(home) = std::env::var_os("BOTLOFT_HOME") else {
        return;
    };
    let path = PathBuf::from(home).join("logs").join("hook.log");
    let line = format!("{} hook {event}: {err}\n", botloft_core::now_ms());
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()));
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;

    /// Serves one request, answers `status` and returns what it received.
    fn one_request(status: &'static str) -> (u16, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            stream
                .set_read_timeout(Some(Duration::from_millis(500)))
                .expect("timeout");
            let mut received = Vec::new();
            let mut buf = [0u8; 4096];
            while let Ok(n) = stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                received.extend_from_slice(&buf[..n]);
                if received.windows(4).any(|w| w == b"\r\n\r\n") && received.ends_with(b"}") {
                    break;
                }
            }
            stream
                .write_all(format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\n\r\n").as_bytes())
                .expect("answer");
            String::from_utf8_lossy(&received).into_owned()
        });
        (port, server)
    }

    #[test]
    fn posts_the_payload_with_the_bot_token() {
        let (port, server) = one_request("204 No Content");
        post(port, "/hooks/stop", "tok", br#"{"payload":{}}"#).expect("post");
        let request = server.join().expect("server");
        assert!(
            request.starts_with("POST /hooks/stop HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(request.contains("Authorization: Bearer tok\r\n"));
        assert!(request.ends_with(r#"{"payload":{}}"#));
    }

    #[test]
    fn a_refusal_is_an_error_without_the_token() {
        let (port, server) = one_request("401 Unauthorized");
        let err = post(port, "/hooks/stop", "secret-token", b"{}").expect_err("refused");
        server.join().expect("server");
        assert!(
            err.contains("401") && !err.contains("secret-token"),
            "{err}"
        );
    }

    #[test]
    fn unknown_events_are_rejected_before_any_network_call() {
        let err = forward("explode", &mut io::empty()).expect_err("unknown");
        assert!(err.contains("explode"));
    }
}
