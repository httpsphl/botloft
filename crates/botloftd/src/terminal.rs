//! A bot's terminal output (spec 8): a ring buffer addressed by byte offset
//! within a process generation, and a broadcast of new output to every
//! attached client.

use std::collections::VecDeque;
use std::sync::Mutex;

use bytes::Bytes;
use tokio::sync::broadcast;

/// Output chunks a slow client may lag behind before it has to re-attach.
const LIVE_BUFFER: usize = 256;

/// Output of one generation, starting at `offset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub generation: u64,
    pub offset: u64,
    pub data: Bytes,
}

/// Where a client picks up after `attach`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachPoint {
    pub generation: u64,
    /// Offset of the first byte of the replay.
    pub offset: u64,
    /// The client must clear its screen: it saw another generation, or the
    /// bytes after its offset are no longer buffered.
    pub reset: bool,
    /// End of the replay, where live output starts.
    pub live_offset: u64,
}

pub struct Attachment {
    pub point: AttachPoint,
    /// Buffered bytes from `point.offset`, possibly empty.
    pub replay: Bytes,
    /// Everything pushed after the replay, with no gap and no overlap.
    pub live: broadcast::Receiver<Chunk>,
}

pub struct Terminal {
    inner: Mutex<Inner>,
    live: broadcast::Sender<Chunk>,
}

struct Inner {
    generation: u64,
    capacity: usize,
    /// Offset of `buf[0]` within the generation.
    start: u64,
    buf: VecDeque<u8>,
}

impl Inner {
    fn end(&self) -> u64 {
        self.start + self.buf.len() as u64
    }
}

impl Terminal {
    pub fn new(capacity: usize) -> Self {
        let (live, _) = broadcast::channel(LIVE_BUFFER);
        Self {
            inner: Mutex::new(Inner {
                generation: 0,
                capacity: capacity.max(1),
                start: 0,
                buf: VecDeque::new(),
            }),
            live,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn generation(&self) -> u64 {
        self.lock().generation
    }

    /// Starts a new generation with an empty buffer.
    pub fn reset(&self, generation: u64) {
        let mut inner = self.lock();
        inner.generation = generation;
        inner.start = 0;
        inner.buf.clear();
    }

    /// Appends output of `generation`; output of an older one is dropped.
    pub fn push(&self, generation: u64, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        let mut inner = self.lock();
        if inner.generation != generation {
            return;
        }
        let offset = inner.end();
        inner.buf.extend(data);
        let excess = inner.buf.len().saturating_sub(inner.capacity);
        if excess > 0 {
            inner.buf.drain(..excess);
            inner.start += excess as u64;
        }
        // Sent under the lock so attach() never sees a chunk twice or misses one.
        let _ = self.live.send(Chunk {
            generation,
            offset,
            data: Bytes::copy_from_slice(data),
        });
    }

    /// Replays from what the client last saw when possible (spec 8).
    pub fn attach(&self, generation: Option<u64>, offset: Option<u64>) -> Attachment {
        let inner = self.lock();
        let live = self.live.subscribe();
        let resumable = match (generation, offset) {
            (Some(g), Some(o)) => g == inner.generation && (inner.start..=inner.end()).contains(&o),
            _ => false,
        };
        let from = if resumable {
            offset.unwrap_or(inner.start)
        } else {
            reset_point(&inner)
        };
        let skip = usize::try_from(from - inner.start).unwrap_or(usize::MAX);
        let replay: Vec<u8> = inner.buf.iter().skip(skip).copied().collect();
        Attachment {
            point: AttachPoint {
                generation: inner.generation,
                offset: from,
                reset: !resumable,
                live_offset: inner.end(),
            },
            replay: Bytes::from(replay),
            live,
        }
    }
}

/// Finds the first cursor position query (`ESC [ 6 n`) in a byte stream,
/// even when it is split across chunks.
///
/// ConPTY, as `portable-pty` creates it (`PSEUDOCONSOLE_INHERIT_CURSOR`),
/// asks the host terminal where the cursor is when it starts and holds the
/// child until it gets an answer. The daemon has no screen, so it answers
/// that first query itself; later queries are left to attached terminals.
#[derive(Debug, Default)]
pub struct CursorQueryWatch {
    matched: usize,
    seen: bool,
}

impl CursorQueryWatch {
    const QUERY: &'static [u8] = b"\x1b[6n";
    /// Row 1, column 1: a fresh screen.
    pub const ANSWER: &'static [u8] = b"\x1b[1;1R";

    /// Feeds output; true exactly once, when the first query completes.
    pub fn feed(&mut self, data: &[u8]) -> bool {
        if self.seen {
            return false;
        }
        for &byte in data {
            self.matched = if byte == Self::QUERY[self.matched] {
                self.matched + 1
            } else if byte == Self::QUERY[0] {
                1
            } else {
                0
            };
            if self.matched == Self::QUERY.len() {
                self.seen = true;
                return true;
            }
        }
        false
    }
}

/// Where a full replay starts: the whole generation when nothing was dropped,
/// otherwise after the first newline so it never opens mid escape sequence.
fn reset_point(inner: &Inner) -> u64 {
    if inner.start == 0 {
        return 0;
    }
    match inner.buf.iter().position(|&b| b == b'\n') {
        Some(newline) => inner.start + newline as u64 + 1,
        None => inner.start,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(bytes: &Bytes) -> &str {
        std::str::from_utf8(bytes).expect("utf8")
    }

    #[test]
    fn replays_only_what_the_client_missed() {
        let term = Terminal::new(1024);
        term.reset(7);
        term.push(7, b"hello ");
        term.push(7, b"world");

        let fresh = term.attach(None, None);
        assert_eq!(
            fresh.point,
            AttachPoint {
                generation: 7,
                offset: 0,
                reset: true,
                live_offset: 11
            }
        );
        assert_eq!(text(&fresh.replay), "hello world");

        let resumed = term.attach(Some(7), Some(6));
        assert_eq!(
            resumed.point,
            AttachPoint {
                generation: 7,
                offset: 6,
                reset: false,
                live_offset: 11
            }
        );
        assert_eq!(text(&resumed.replay), "world");

        let up_to_date = term.attach(Some(7), Some(11));
        assert!(!up_to_date.point.reset);
        assert!(up_to_date.replay.is_empty());
    }

    #[test]
    fn another_generation_or_a_dropped_offset_resets() {
        let term = Terminal::new(8);
        term.reset(1);
        term.push(1, b"ab\ncdefgh\nij");
        // Only the last 8 bytes stay: "defgh\nij".
        let old = term.attach(Some(1), Some(0));
        assert!(old.point.reset);
        assert_eq!(text(&old.replay), "ij", "cut after the first newline");

        term.reset(2);
        term.push(2, b"new");
        let stale = term.attach(Some(1), Some(3));
        assert_eq!(
            stale.point,
            AttachPoint {
                generation: 2,
                offset: 0,
                reset: true,
                live_offset: 3
            }
        );
        assert_eq!(text(&stale.replay), "new");
    }

    #[test]
    fn the_first_cursor_query_is_found_once_even_when_split() {
        let mut watch = CursorQueryWatch::default();
        assert!(!watch.feed(b"banner \x1b["));
        assert!(watch.feed(b"6n more"));
        assert!(!watch.feed(b"\x1b[6n"), "only the first one");

        let mut near_miss = CursorQueryWatch::default();
        assert!(!near_miss.feed(b"\x1b[5n\x1b[?25l"));
        assert!(near_miss.feed(b"\x1b\x1b[6n"));
    }

    #[test]
    fn live_output_follows_the_replay_without_gaps() {
        let term = Terminal::new(64);
        term.reset(3);
        term.push(3, b"one");
        let mut attached = term.attach(None, None);
        term.push(3, b"two");
        term.push(2, b"stale generation is dropped");

        let chunk = attached.live.try_recv().expect("live chunk");
        assert_eq!(
            chunk.offset,
            attached.point.offset + attached.replay.len() as u64
        );
        assert_eq!(&chunk.data[..], b"two");
        assert!(attached.live.try_recv().is_err());
    }
}
