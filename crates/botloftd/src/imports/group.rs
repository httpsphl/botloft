//! Joining repeated requests (spec 36.5): the same request, case,
//! punctuation and spacing aside, becomes one line with how many times it
//! was made, on which days of the week and at what hour, the best hint of a
//! routine. Used for Claude Code's requests and claude.ai's messages alike.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use jiff::Timestamp;
use jiff::tz::TimeZone;

use super::words::Words;

/// One request, with every time it was made.
pub struct Group {
    /// In its newest wording.
    pub text: String,
    pub count: u32,
    days: [u32; 7],
    hours: [u32; 24],
}

/// What makes two requests the same: case, punctuation and spacing aside.
pub fn same_key(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `at` (Unix ms) as a date in `tz`.
pub fn day(at: i64, tz: &TimeZone) -> String {
    Timestamp::from_millisecond(at)
        .map(|at| at.to_zoned(tz.clone()).strftime("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// `items` (text, Unix ms), newest first, joined; at most `max` groups.
pub fn groups(
    items: impl IntoIterator<Item = (String, i64)>,
    tz: &TimeZone,
    max: usize,
) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    for (text, at) in items {
        let key = same_key(&text);
        if key.is_empty() {
            continue;
        }
        let slot = match index.get(&key) {
            Some(&slot) => slot,
            None => {
                if groups.len() >= max {
                    continue;
                }
                index.insert(key, groups.len());
                groups.push(Group {
                    text,
                    count: 0,
                    days: [0; 7],
                    hours: [0; 24],
                });
                groups.len() - 1
            }
        };
        let group = &mut groups[slot];
        group.count += 1;
        if let Ok(at) = Timestamp::from_millisecond(at).map(|at| at.to_zoned(tz.clone())) {
            group.days[at.weekday().to_monday_zero_offset() as usize] += 1;
            group.hours[at.hour() as usize] += 1;
        }
    }
    groups
}

/// "14 vezes, seg/qua/sex ~9h" for a request made more than once.
pub fn when(group: &Group, words: &Words) -> String {
    let mut days: Vec<(usize, u32)> = group.days.iter().copied().enumerate().collect();
    days.retain(|(_, count)| *count > 0);
    days.sort_by_key(|(day, count)| (std::cmp::Reverse(*count), *day));
    days.truncate(3);
    days.sort_by_key(|(day, _)| *day);
    let names: Vec<&str> = days.iter().map(|(day, _)| words.days[*day]).collect();
    let hour = (0..24).max_by_key(|hour| (group.hours[*hour], std::cmp::Reverse(*hour)));
    let mut out = format!("{} {}", group.count, words.times);
    if !names.is_empty() {
        let _ = write!(out, ", {}", names.join("/"));
    }
    if let Some(hour) = hour.filter(|hour| group.hours[*hour] > 0) {
        let _ = write!(out, " ~{hour}h");
    }
    out
}
