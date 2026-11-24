//! The agents' only way out of their box (spec 42.5): an HTTP proxy that
//! resolves names itself and dials only the public internet (`address`).
//! The boxes reach it through a unix socket, and the agents' browser
//! through a port on loopback. TLS goes through `CONNECT`; plain HTTP comes
//! as a request for an absolute URL.

pub mod address;

use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::debug;

use self::address::Range;

/// The longest request head read before the request is refused.
const HEAD_MAX: usize = 16 * 1024;
/// How long a client may take to send its head, and the gate to resolve a
/// name or reach the address.
const WAIT: Duration = Duration::from_secs(20);

/// The gate's port for the agents' browsers, once it is open.
static BROWSERS: OnceLock<u16> = OnceLock::new();

/// From now on the agents' browsers go out through the gate on `port`.
pub fn use_for_browsers(port: u16) {
    let _ = BROWSERS.set(port);
}

/// The port the agents' browsers go out through, when they have a gate.
pub fn for_browsers() -> Option<u16> {
    BROWSERS.get().copied()
}

pub struct Gate {
    reach: Vec<Range>,
}

impl Gate {
    /// `reach`: the addresses and ranges the owner opened (`reach` in the
    /// settings); what cannot be read is left out.
    pub fn new(reach: &[String]) -> Arc<Self> {
        let reach = reach.iter().filter_map(|text| Range::parse(text)).collect();
        Arc::new(Self { reach })
    }

    /// Why `address` is refused, read against this computer's addresses as
    /// they are now (a server's address may change).
    fn refusal(&self, address: IpAddr) -> Option<&'static str> {
        address::refusal(address, &crate::platform::own_addresses(), &self.reach)
    }
}

/// Serves the browsers' port until the daemon stops.
pub async fn serve_tcp(listener: TcpListener, gate: Arc<Gate>) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        tokio::spawn(handle(stream, Arc::clone(&gate)));
    }
}

/// Serves the boxes' socket until the daemon stops.
#[cfg(unix)]
pub async fn serve_unix(listener: tokio::net::UnixListener, gate: Arc<Gate>) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        tokio::spawn(handle(stream, Arc::clone(&gate)));
    }
}

/// What a client asked for.
#[derive(Debug, PartialEq, Eq)]
struct Asked {
    host: String,
    port: u16,
    /// `None` for `CONNECT`; for plain HTTP, the head to send on, with the
    /// path in place of the URL.
    forward: Option<Vec<u8>>,
}

async fn handle<S: AsyncRead + AsyncWrite + Unpin>(mut client: S, gate: Arc<Gate>) {
    let Ok(Ok((head, rest))) = tokio::time::timeout(WAIT, read_head(&mut client)).await else {
        return;
    };
    let Some(asked) = parse(&head) else {
        let _ = client.write_all(&answer(400, "not a proxy request")).await;
        return;
    };
    let upstream = match tokio::time::timeout(WAIT, dial(&gate, &asked)).await {
        Ok(Ok(upstream)) => upstream,
        Ok(Err(reason)) => {
            debug!(port = asked.port, reason, "the gate refused");
            let _ = client.write_all(&answer(403, reason)).await;
            return;
        }
        Err(_) => {
            let _ = client.write_all(&answer(504, "no answer")).await;
            return;
        }
    };
    let mut upstream = upstream;
    let sent = match &asked.forward {
        None => client
            .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
            .await
            .is_ok(),
        Some(head) => upstream.write_all(head).await.is_ok(),
    };
    if !sent || (!rest.is_empty() && upstream.write_all(&rest).await.is_err()) {
        return;
    }
    let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
}

/// The first address of `asked` the gate may dial, connected.
async fn dial(gate: &Gate, asked: &Asked) -> Result<TcpStream, &'static str> {
    let addresses: Vec<SocketAddr> = match asked.host.parse::<IpAddr>() {
        Ok(address) => vec![SocketAddr::new(address, asked.port)],
        Err(_) => tokio::net::lookup_host((asked.host.as_str(), asked.port))
            .await
            .map_err(|_| "the name does not resolve")?
            .collect(),
    };
    let mut reason = "the name does not resolve";
    for address in addresses {
        if let Some(refused) = gate.refusal(address.ip()) {
            reason = refused;
            continue;
        }
        // The address checked is the one dialled: a name that resolves
        // again elsewhere cannot slip past.
        if let Ok(stream) = TcpStream::connect(address).await {
            return Ok(stream);
        }
        reason = "unreachable";
    }
    Err(reason)
}

/// Reads up to the end of the head; what came after it is the body's start
/// (or, after `CONNECT`, the first bytes of TLS).
async fn read_head<S: AsyncRead + Unpin>(client: &mut S) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 2048];
    loop {
        let read = client.read(&mut chunk).await?;
        if read == 0 {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            let rest = buffer.split_off(end + 4);
            return Ok((buffer, rest));
        }
        if buffer.len() > HEAD_MAX {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
    }
}

fn parse(head: &[u8]) -> Option<Asked> {
    let text = std::str::from_utf8(head).ok()?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target, version) = (first.next()?, first.next()?, first.next()?);
    if method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) = host_port(target, None)?;
        return Some(Asked {
            host,
            port,
            forward: None,
        });
    }
    let rest = target.strip_prefix("http://")?;
    let (authority, path) = match rest.find('/') {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, "/"),
    };
    let (host, port) = host_port(authority, Some(80))?;
    let mut forward = format!("{method} {path} {version}\r\n");
    for line in lines.filter(|line| !line.is_empty()) {
        let name = line.split(':').next().unwrap_or("").trim();
        let hop = [
            "proxy-connection",
            "proxy-authorization",
            "connection",
            "keep-alive",
        ];
        if !hop.iter().any(|hop| name.eq_ignore_ascii_case(hop)) {
            forward.push_str(line);
            forward.push_str("\r\n");
        }
    }
    // One request per connection: the next could name another host.
    forward.push_str("Connection: close\r\n\r\n");
    Some(Asked {
        host,
        port,
        forward: Some(forward.into_bytes()),
    })
}

/// `example.com:443`, `[2001:db8::1]:443`, or a host alone with `default`.
fn host_port(authority: &str, default: Option<u16>) -> Option<(String, u16)> {
    let authority = authority.rsplit('@').next()?;
    let (host, port) = if let Some(inner) = authority.strip_prefix('[') {
        let (host, after) = inner.split_once(']')?;
        let port = match after.strip_prefix(':') {
            Some(port) => port.parse().ok()?,
            None if after.is_empty() => default?,
            None => return None,
        };
        (host, port)
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) => (host, port.parse().ok()?),
            None => (authority, default?),
        }
    };
    (!host.is_empty() && port != 0).then(|| (host.to_ascii_lowercase(), port))
}

fn answer(status: u16, reason: &str) -> Vec<u8> {
    let title = match status {
        400 => "Bad Request",
        403 => "Forbidden",
        _ => "Gateway Timeout",
    };
    let body = format!("Botloft's gate: {reason}\n");
    format!(
        "HTTP/1.1 {status} {title}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

#[cfg(test)]
mod tests;
