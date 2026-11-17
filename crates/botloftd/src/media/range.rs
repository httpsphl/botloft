//! The piece of a file a player asks for, from its `Range` header (RFC 9110,
//! 14.2): one span of bytes. Anything else gets the whole file, which the
//! RFC allows a server to answer with.

/// What a request gets of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    Whole,
    /// First and last byte, both included.
    Part {
        first: u64,
        last: u64,
    },
    /// It starts past the end of the file: 416.
    Outside,
}

/// Digits only: `u64::from_str` would also take a `+`.
fn number(text: &str) -> Option<u64> {
    (!text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

/// The span `header` asks for in a file of `len` bytes.
pub fn span(header: Option<&str>, len: u64) -> Span {
    let Some(spec) = header.and_then(|value| value.trim().strip_prefix("bytes=")) else {
        return Span::Whole;
    };
    // Several spans would need a multipart answer.
    let Some((first, last)) = spec.split_once('-').filter(|_| !spec.contains(',')) else {
        return Span::Whole;
    };
    let (first, last) = (first.trim(), last.trim());
    if first.is_empty() {
        // `-n`: the last n bytes.
        return match number(last) {
            None => Span::Whole,
            Some(0) => Span::Outside,
            Some(_) if len == 0 => Span::Outside,
            Some(count) => Span::Part {
                first: len.saturating_sub(count),
                last: len - 1,
            },
        };
    }
    let Some(first) = number(first) else {
        return Span::Whole;
    };
    let last = if last.is_empty() {
        None
    } else {
        match number(last) {
            Some(last) if last >= first => Some(last),
            _ => return Span::Whole,
        }
    };
    if first >= len {
        return Span::Outside;
    }
    Span::Part {
        first,
        last: last.map_or(len - 1, |last| last.min(len - 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(first: u64, last: u64) -> Span {
        Span::Part { first, last }
    }

    #[test]
    fn one_span_is_served_and_the_rest_gets_the_whole_file() {
        assert_eq!(span(None, 100), Span::Whole);
        assert_eq!(span(Some("bytes=0-"), 100), part(0, 99));
        assert_eq!(span(Some("bytes=10-19"), 100), part(10, 19));
        assert_eq!(span(Some("bytes=90-500"), 100), part(90, 99));
        assert_eq!(span(Some("bytes=-30"), 100), part(70, 99));
        assert_eq!(span(Some("bytes=-500"), 100), part(0, 99));
        assert_eq!(span(Some(" bytes= 5 - 6 "), 100), part(5, 6));

        assert_eq!(span(Some("bytes=100-"), 100), Span::Outside);
        assert_eq!(span(Some("bytes=-0"), 100), Span::Outside);
        assert_eq!(span(Some("bytes=0-"), 0), Span::Outside);
        assert_eq!(span(Some("bytes=-5"), 0), Span::Outside);

        for ignored in [
            "items=0-5",
            "bytes=0-5,10-20",
            "bytes=9-3",
            "bytes=+1-5",
            "bytes=a-b",
            "bytes=5",
            "bytes=-",
        ] {
            assert_eq!(span(Some(ignored), 100), Span::Whole, "{ignored}");
        }
    }
}
