//! `botloftd box-bridge`, the first program in an agent's box (spec 42.5).
//! The box's network is a loopback of its own; this listens there on the
//! ports the agent knows (Botloft's MCP, the gate) and carries each
//! connection to a unix socket that bubblewrap brought in from outside.
//! Then it runs the agent and ends with it.

use std::ffi::OsString;
use std::io;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

/// `127.0.0.1:45710=/run/botloft/mcp.sock`: a port inside, a socket out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bridge {
    pub listen: SocketAddr,
    pub socket: PathBuf,
}

impl Bridge {
    pub fn parse(text: &str) -> Option<Self> {
        let (listen, socket) = text.split_once('=')?;
        Some(Self {
            listen: listen.parse().ok()?,
            socket: PathBuf::from(socket),
        })
    }
}

/// Opens the bridges, runs `command` and returns its exit code.
pub fn run(bridges: &[Bridge], command: &[OsString]) -> io::Result<i32> {
    let Some((program, args)) = command.split_first() else {
        return Err(io::Error::other("no program to run"));
    };
    for bridge in bridges {
        let listener = TcpListener::bind(bridge.listen)?;
        let socket = bridge.socket.clone();
        thread::Builder::new()
            .name("box-bridge".into())
            .spawn(move || serve(&listener, &socket))?;
    }
    let status = Command::new(program).args(args).status()?;
    Ok(status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)))
}

fn serve(listener: &TcpListener, socket: &Path) {
    for client in listener.incoming().flatten() {
        let socket = socket.to_owned();
        let _ = thread::Builder::new()
            .name("box-bridge-conn".into())
            .spawn(move || carry(client, &socket));
    }
}

/// Copies each way until both ends are done.
fn carry(client: TcpStream, socket: &Path) {
    let Ok(outside) = UnixStream::connect(socket) else {
        let _ = client.shutdown(Shutdown::Both);
        return;
    };
    let (Ok(mut client_in), Ok(mut outside_out)) = (client.try_clone(), outside.try_clone()) else {
        return;
    };
    let up = thread::spawn(move || {
        let _ = io::copy(&mut client_in, &mut outside_out);
        let _ = outside_out.shutdown(Shutdown::Write);
    });
    let (mut client_out, mut outside_in) = (client, outside);
    let _ = io::copy(&mut outside_in, &mut client_out);
    let _ = client_out.shutdown(Shutdown::Write);
    let _ = up.join();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;

    #[test]
    fn a_bridge_is_read_from_its_text() {
        assert_eq!(
            Bridge::parse("127.0.0.1:45710=/run/botloft/mcp.sock"),
            Some(Bridge {
                listen: "127.0.0.1:45710".parse().expect("addr"),
                socket: "/run/botloft/mcp.sock".into(),
            })
        );
        assert_eq!(Bridge::parse("45710=/x"), None);
        assert_eq!(Bridge::parse("127.0.0.1:1"), None);
    }

    #[test]
    fn a_connection_on_the_port_reaches_the_socket() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("echo.sock");
        let unix = UnixListener::bind(&path).expect("socket");
        thread::spawn(move || {
            let (mut stream, _) = unix.accept().expect("accept");
            let mut asked = [0u8; 4];
            stream.read_exact(&mut asked).expect("read");
            stream.write_all(b"pong").expect("write");
        });
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        thread::spawn(move || serve(&listener, &path));
        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("connect");
        client.write_all(b"ping").expect("ask");
        let mut said = String::new();
        client.read_to_string(&mut said).expect("answer");
        assert_eq!(said, "pong");
    }

    #[test]
    fn the_agent_runs_and_its_exit_code_comes_back() {
        let code = run(&[], &["sh".into(), "-c".into(), "exit 7".into()]).expect("run");
        assert_eq!(code, 7);
    }
}
