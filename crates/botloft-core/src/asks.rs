//! Whether a bot's reply ends with a question, so that a 👍 or 👎 on it
//! answers that question at once (spec 8.9). The app has the same rule in
//! `app/src/features/chat/asks.ts`.

/// The reactions that answer a question, and what each one says.
pub const ANSWERS: [(&str, &str); 2] = [("👍", "yes"), ("👎", "no")];

/// What can close a line after its question mark: emphasis, code, quotes
/// and brackets.
const CLOSERS: &[char] = &['*', '_', '`', '"', '\'', '”', '’', '»', ')', ']'];

/// Where a sentence ends, when a space follows.
const STOPS: &[char] = &['.', '!', '?', '？', '！'];

/// What can open a line: list marks, quotes and headings.
const OPENERS: &[char] = &['-', '*', '+', '>', '#', '_', '`', '"', '“', '«', '('];

fn asks(end: &str) -> bool {
    end.ends_with('?') || end.ends_with('？')
}

/// The question `text` ends with: the last sentence of its last line, on
/// one line, at most `max` characters, keeping its end. `None` when the
/// last line asks nothing.
pub fn asked(text: &str, max: usize) -> Option<String> {
    let line = text.lines().map(str::trim).rfind(|line| !line.is_empty())?;
    let end = line.trim_end_matches(|c: char| c.is_whitespace() || CLOSERS.contains(&c));
    if !asks(end) {
        return None;
    }
    let mark = end.chars().next_back().map_or(0, char::len_utf8);
    let before = &end[..end.len() - mark];
    let mut start = 0;
    let mut chars = before.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if STOPS.contains(&c) && chars.peek().is_some_and(|(_, next)| next.is_whitespace()) {
            start = at + c.len_utf8();
        }
    }
    let sentence =
        end[start..].trim_start_matches(|c: char| c.is_whitespace() || OPENERS.contains(&c));
    let flat = sentence.split_whitespace().collect::<Vec<_>>().join(" ");
    let count = flat.chars().count();
    if count <= max {
        return Some(flat);
    }
    let kept: String = flat.chars().skip(count + 1 - max).collect();
    Some(format!("…{}", kept.trim_start()))
}

/// What a reaction says to a question it answers: `yes` for 👍 and `no`
/// for 👎 on a quote that asks. `None` for any other reaction.
pub fn answer_word(emoji: &str, quote: &str) -> Option<&'static str> {
    if !asks(quote) {
        return None;
    }
    ANSWERS
        .iter()
        .find(|(answer, _)| *answer == emoji)
        .map(|(_, word)| *word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_is_the_last_sentence_of_the_last_line() {
        assert_eq!(
            asked(
                "The report is ready.\n\nI checked the figures. Should I send it to Acme?",
                120
            )
            .as_deref(),
            Some("Should I send it to Acme?")
        );
        assert_eq!(
            asked("Done.\n\n**Want me to deploy v2.1 now?**\n", 120).as_deref(),
            Some("Want me to deploy v2.1 now?")
        );
        assert_eq!(
            asked("- Pode publicar? ", 120).as_deref(),
            Some("Pode publicar?")
        );
        assert_eq!(
            asked("¿Lo mando ya？", 120).as_deref(),
            Some("¿Lo mando ya？")
        );
    }

    #[test]
    fn a_reply_that_asks_nothing_last_has_no_question() {
        assert_eq!(asked("Should I? I already did it.", 120), None);
        assert_eq!(asked("Is it ready?\n\nYes, it is.", 120), None);
        assert_eq!(asked("", 120), None);
        assert_eq!(asked("\n  \n", 120), None);
    }

    #[test]
    fn a_long_question_keeps_its_end() {
        let long = format!("Should I {} now?", "really ".repeat(40));
        let quoted = asked(&long, 40).expect("asks");
        assert!(quoted.chars().count() <= 40, "{quoted}");
        assert!(quoted.starts_with('…') && quoted.ends_with("really now?"));
    }

    #[test]
    fn only_a_thumb_on_a_question_is_an_answer() {
        assert_eq!(answer_word("👍", "Deploy now?"), Some("yes"));
        assert_eq!(answer_word("👎", "Deploy now?"), Some("no"));
        assert_eq!(answer_word("❤️", "Deploy now?"), None);
        assert_eq!(answer_word("👍", "Deployed."), None);
    }
}
