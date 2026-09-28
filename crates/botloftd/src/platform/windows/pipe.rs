//! Writing to a Claude Code inbox, a named pipe the bot's own Claude Code
//! process serves (spec 9.2). The daemon is the client.

use std::io;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::time::Instant;
use windows::Win32::Foundation::ERROR_PIPE_BUSY;

/// While every instance of the pipe is taken, try again this often...
const BUSY_RETRY: Duration = Duration::from_millis(50);
/// ...for this long, within one delivery attempt.
const BUSY_FOR: Duration = Duration::from_secs(2);

/// Opens the pipe at `address`, writes `payload` and closes it. What was
/// written stays readable for Claude Code after the close.
pub async fn write_inbox(address: &str, payload: &[u8]) -> io::Result<()> {
    if !is_pipe_name(address) {
        // The address comes from a bot's hook. Anything but a plain pipe
        // name could point `CreateFile` at a real file.
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the inbox address is not a named pipe",
        ));
    }
    let give_up = Instant::now() + BUSY_FOR;
    let mut pipe = loop {
        match ClientOptions::new().open(address) {
            Ok(pipe) => break pipe,
            Err(err)
                if err.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32)
                    && Instant::now() < give_up =>
            {
                tokio::time::sleep(BUSY_RETRY).await;
            }
            Err(err) => return Err(err),
        }
    };
    pipe.write_all(payload).await?;
    pipe.flush().await
}

/// `\\.\pipe\` followed by plain name segments: no `.` or `..` segment,
/// which Windows would resolve to a path outside the pipe namespace.
fn is_pipe_name(address: &str) -> bool {
    const PREFIX: &str = r"\\.\pipe\";
    let Some(name) = address
        .get(..PREFIX.len())
        .filter(|prefix| prefix.eq_ignore_ascii_case(PREFIX))
        .and_then(|_| address.get(PREFIX.len()..))
    else {
        return false;
    };
    !name.is_empty()
        && name.split('\\').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;
    use tokio::net::windows::named_pipe::ServerOptions;

    use super::*;

    fn unique_name() -> String {
        format!(
            r"\\.\pipe\LOCAL\botloft-test-{}",
            botloft_core::ids::MessageId::generate().as_str()
        )
    }

    #[test]
    fn only_plain_pipe_names_are_accepted() {
        assert!(is_pipe_name(
            r"\\.\pipe\LOCAL\cc-msg-0123456789abcdef0123456789abcdef"
        ));
        assert!(is_pipe_name(r"\\.\PIPE\claude.1"));
        for bad in [
            r"C:\Users\ana\notes.txt",
            r"\\.\pipe\",
            r"\\.\pipe\..\C:\Windows\win.ini",
            r"\\.\pipe\LOCAL\..\x",
            r"\\.\pipe\a/b",
            r"\\.\pipe\a:b",
            r"\\?\pipe\x",
        ] {
            assert!(!is_pipe_name(bad), "{bad}");
        }
    }

    #[tokio::test]
    async fn writes_everything_and_closes() {
        let name = unique_name();
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&name)
            .expect("server");
        let reader = tokio::spawn(async move {
            server.connect().await.expect("connect");
            let mut received = Vec::new();
            server.read_to_end(&mut received).await.expect("read");
            received
        });
        write_inbox(&name, b"line 1\nline 2\n")
            .await
            .expect("write");
        assert_eq!(reader.await.expect("reader"), b"line 1\nline 2\n");
    }

    #[tokio::test]
    async fn waits_while_the_pipe_is_busy() {
        let name = unique_name();
        let first = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&name)
            .expect("server");
        // Someone else holds the only instance.
        let _other = ClientOptions::new().open(&name).expect("other client");
        let name_for_server = name.clone();
        let late = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(150)).await;
            let mut second = ServerOptions::new()
                .create(&name_for_server)
                .expect("second instance");
            second.connect().await.expect("connect");
            let mut received = Vec::new();
            second.read_to_end(&mut received).await.expect("read");
            received
        });
        write_inbox(&name, b"hello\n").await.expect("write");
        assert_eq!(late.await.expect("server"), b"hello\n");
        drop(first);
    }

    #[tokio::test]
    async fn a_missing_pipe_fails() {
        assert!(write_inbox(&unique_name(), b"x\n").await.is_err());
    }
}
