//! Swapping out what must not go to the Maker (spec 36.7): keys and
//! passwords, the exact values of the agents' keys (spec 32), e-mails,
//! phone, CPF and CNPJ numbers, and card numbers that pass the Luhn check.
//! It counts what it swapped, by kind. It cannot be perfect, and the
//! screen that shows the summary says so.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::{Captures, Regex};

/// What a swapped value becomes, in the owner's language.
#[derive(Debug, Clone, Copy)]
pub struct Labels {
    pub key: &'static str,
    pub email: &'static str,
    pub number: &'static str,
    pub card: &'static str,
}

impl Labels {
    pub fn for_locale(locale: &str) -> Self {
        if locale.starts_with("pt") {
            Self {
                key: "[chave]",
                email: "[e-mail]",
                number: "[número]",
                card: "[cartão]",
            }
        } else if locale.starts_with("es") {
            Self {
                key: "[clave]",
                email: "[correo]",
                number: "[número]",
                card: "[tarjeta]",
            }
        } else {
            Self {
                key: "[key]",
                email: "[email]",
                number: "[number]",
                card: "[card]",
            }
        }
    }
}

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a valid pattern")
}

/// Keys and passwords, by their shape. A label (`password=`) stays and only
/// its value goes, in the group `value`.
static KEYS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"-----BEGIN [A-Z ]+-----[\s\S]*?-----END [A-Z ]+-----",
        r"\bsk-ant-[A-Za-z0-9_\-]{10,}",
        r"\bsk-[A-Za-z0-9_\-]{16,}",
        r"\bgh[pousr]_[A-Za-z0-9]{20,}",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}",
        r"\bxox[abpors]-[A-Za-z0-9\-]{10,}",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\bAIza[0-9A-Za-z_\-]{30,}",
        r"\beyJ[A-Za-z0-9_\-]{8,}\.eyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}",
        r"(?i)\bbearer\s+(?P<value>[A-Za-z0-9._~+/=\-]{12,})",
        r#"(?i)\b(?:password|passwd|pwd|senha|contraseña|secret|token|api[_-]?key)\s*[:=]\s*(?P<value>[^\s"',;]{4,})"#,
        r"\b[0-9a-fA-F]{32,}\b",
    ]
    .into_iter()
    .map(regex)
    .collect()
});

/// A long run that could be base64: a key when it mixes letters and digits.
static LONG: LazyLock<Regex> = LazyLock::new(|| regex(r"[A-Za-z0-9+/_\-]{40,}={0,2}"));

static EMAIL: LazyLock<Regex> =
    LazyLock::new(|| regex(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b"));
static CARD: LazyLock<Regex> = LazyLock::new(|| regex(r"\b(?:\d[ \-]?){12,18}\d\b"));
static CPF_CNPJ: LazyLock<Regex> = LazyLock::new(|| {
    regex(
        r"\b\d{3}\.\d{3}\.\d{3}-\d{2}\b|\b\d{2}\.\d{3}\.\d{3}/\d{4}-\d{2}\b|\b\d{11}\b|\b\d{14}\b",
    )
});
static PHONE: LazyLock<Regex> =
    LazyLock::new(|| regex(r"(?:\+\d{1,3}[\s.\-]?)?\(?\b\d{2,3}\)?[\s.\-]?\d{4,5}[\s.\-]\d{4}\b"));

/// Whether `digits` passes the Luhn check that card numbers carry.
fn luhn(digits: &str) -> bool {
    let numbers: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&numbers.len()) {
        return false;
    }
    let sum: u32 = numbers
        .iter()
        .rev()
        .enumerate()
        .map(|(index, &digit)| {
            if index % 2 == 1 {
                let doubled = digit * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                digit
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// Swaps out what must not go, counting it by kind.
pub struct Redactor {
    labels: Labels,
    /// The agents' key values, longest first, so one inside another goes whole.
    exact: Vec<String>,
}

impl Redactor {
    pub fn new(labels: Labels, mut exact: Vec<String>) -> Self {
        exact.retain(|value| value.chars().count() >= 6);
        exact.sort_by_key(|value| std::cmp::Reverse(value.len()));
        exact.dedup();
        Self { labels, exact }
    }

    /// `text` with what must not go swapped out; `counts` grows by kind.
    pub fn clean(&self, text: &str, counts: &mut BTreeMap<String, u32>) -> String {
        let mut add = |kind: &str| *counts.entry(kind.to_owned()).or_default() += 1;
        let mut out = text.to_owned();
        for value in &self.exact {
            let found = out.matches(value.as_str()).count();
            if found > 0 {
                out = out.replace(value.as_str(), self.labels.key);
                for _ in 0..found {
                    add("key");
                }
            }
        }
        for pattern in KEYS.iter() {
            out = pattern
                .replace_all(&out, |caps: &Captures<'_>| {
                    add("key");
                    match caps.name("value") {
                        Some(value) => {
                            let whole = &caps[0];
                            let start = value.start() - caps.get(0).map_or(0, |m| m.start());
                            format!("{}{}", &whole[..start], self.labels.key)
                        }
                        None => self.labels.key.to_owned(),
                    }
                })
                .into_owned();
        }
        out = LONG
            .replace_all(&out, |caps: &Captures<'_>| {
                let run = &caps[0];
                let mixed = run.chars().any(|c| c.is_ascii_digit())
                    && run.chars().any(|c| c.is_ascii_alphabetic());
                if mixed {
                    add("key");
                    self.labels.key.to_owned()
                } else {
                    run.to_owned()
                }
            })
            .into_owned();
        out = EMAIL
            .replace_all(&out, |_: &Captures<'_>| {
                add("email");
                self.labels.email
            })
            .into_owned();
        out = CARD
            .replace_all(&out, |caps: &Captures<'_>| {
                if luhn(&caps[0]) {
                    add("card");
                    self.labels.card.to_owned()
                } else {
                    caps[0].to_owned()
                }
            })
            .into_owned();
        for pattern in [&*CPF_CNPJ, &*PHONE] {
            out = pattern
                .replace_all(&out, |_: &Captures<'_>| {
                    add("number");
                    self.labels.number
                })
                .into_owned();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean(text: &str) -> (String, BTreeMap<String, u32>) {
        let redactor = Redactor::new(
            Labels::for_locale("pt-BR"),
            vec!["agent-key-value-123".to_owned()],
        );
        let mut counts = BTreeMap::new();
        let out = redactor.clean(text, &mut counts);
        (out, counts)
    }

    #[test]
    fn swaps_keys_and_passwords_keeping_their_labels() {
        let (out, counts) = clean(
            "use sk-ant-api03-abcdefghijklmnop and ghp_abcdefghijklmnopqrstuvwxyz12 \
             with senha: hunter22 and Bearer abcdefghijklmnopqrstu; also agent-key-value-123",
        );
        assert!(!out.contains("sk-ant"), "{out}");
        assert!(!out.contains("ghp_"), "{out}");
        assert!(!out.contains("hunter22"), "{out}");
        assert!(!out.contains("agent-key-value-123"), "{out}");
        assert!(out.contains("senha: [chave]"), "{out}");
        assert!(out.contains("Bearer [chave]"), "{out}");
        assert_eq!(counts.get("key"), Some(&5));
    }

    #[test]
    fn swaps_emails_numbers_and_cards_that_pass_luhn() {
        let (out, counts) = clean(
            "write to ana@example.com, call +55 11 91234-5678, CPF 123.456.789-09, \
             card 4111 1111 1111 1111, order 1234 5678 9012 3456",
        );
        assert_eq!(
            out,
            "write to [e-mail], call [número], CPF [número], card [cartão], order 1234 5678 9012 3456"
        );
        assert_eq!(counts.get("email"), Some(&1));
        assert_eq!(counts.get("card"), Some(&1));
        assert_eq!(counts.get("number"), Some(&2));
    }

    #[test]
    fn leaves_plain_words_and_short_numbers() {
        let (out, counts) = clean("Resuma os e-mails de hoje às 9h, versão 2.1.284, 30 dias");
        assert_eq!(
            out,
            "Resuma os e-mails de hoje às 9h, versão 2.1.284, 30 dias"
        );
        assert!(counts.is_empty());
    }
}
