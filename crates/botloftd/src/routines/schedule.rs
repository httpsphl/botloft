//! When a routine runs next (spec 20.2). Calendar schedules (weekly, cron)
//! follow wall-clock time in the routine's zone: a time that does not exist
//! (clocks jump forward) runs at the first instant after the jump, and a
//! time that happens twice (clocks go back) runs once, the first time.
//! Intervals count from the last scheduled time and ignore the zone.

use botloft_core::protocol::Schedule;
use jiff::Timestamp;
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, Offset, TimeZone};

use super::cron::Cron;

pub const INTERVAL_MIN_MINUTES: u32 = 5;
pub const INTERVAL_MAX_MINUTES: u32 = 10_080;
/// Closest two runs may be, also for cron.
const MIN_SPACING_MS: i64 = 5 * 60 * 1000;
/// Occurrences checked for that spacing.
const SPACING_SAMPLES: usize = 20;
/// How far ahead a calendar schedule is searched: `0 0 29 2 *` runs every
/// four years, or eight across a skipped leap year.
const SEARCH_DAYS: i64 = 366 * 8 + 2;

/// Why a schedule was refused: a code the app words in the owner's
/// language (spec 20.8), and an English sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub reason: &'static str,
    pub message: String,
}

impl Problem {
    fn new(reason: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// A schedule checked and ready to compute times from.
#[derive(Debug, Clone)]
pub struct Plan {
    kind: Kind,
    tz: TimeZone,
}

#[derive(Debug, Clone)]
enum Kind {
    Weekly { days: u8, time: Time },
    Interval { ms: i64 },
    Cron(Cron),
}

impl Plan {
    /// Checks the schedule and the zone.
    pub fn new(schedule: &Schedule, timezone: &str) -> Result<Self, Problem> {
        let tz = TimeZone::get(timezone).map_err(|_| {
            Problem::new(
                "timezone_unknown",
                format!("\"{timezone}\" is not a known time zone"),
            )
        })?;
        let kind = match schedule {
            Schedule::Weekly { days, time } => {
                if days.is_empty() {
                    return Err(Problem::new(
                        "days_empty",
                        "pick at least one day of the week",
                    ));
                }
                let mut set = 0u8;
                for day in days {
                    if !(1..=7).contains(day) {
                        return Err(Problem::new(
                            "days_range",
                            "days of the week go from 1 (Monday) to 7 (Sunday)",
                        ));
                    }
                    set |= 1 << day;
                }
                Kind::Weekly {
                    days: set,
                    time: clock_time(time)?,
                }
            }
            Schedule::Interval { minutes } => {
                if !(INTERVAL_MIN_MINUTES..=INTERVAL_MAX_MINUTES).contains(minutes) {
                    return Err(Problem::new(
                        "interval_range",
                        format!(
                            "an interval goes from {INTERVAL_MIN_MINUTES} minutes to one week \
                             ({INTERVAL_MAX_MINUTES} minutes)"
                        ),
                    ));
                }
                Kind::Interval {
                    ms: i64::from(*minutes) * 60_000,
                }
            }
            Schedule::Cron { expr } => {
                Kind::Cron(Cron::parse(expr).map_err(|err| Problem::new("cron_invalid", err))?)
            }
        };
        Ok(Self { kind, tz })
    }

    /// Checks that runs are never closer than five minutes, from `now` on.
    pub fn check_spacing(&self, now: i64) -> Result<(), Problem> {
        let mut last = self.next_after(now, now);
        let mut samples = 0;
        while let Some(at) = last
            && samples < SPACING_SAMPLES
        {
            let next = self.next_after(at, at);
            if let Some(next) = next
                && next - at < MIN_SPACING_MS
            {
                return Err(Problem::new(
                    "too_often",
                    "runs must be at least 5 minutes apart",
                ));
            }
            last = next;
            samples += 1;
        }
        if samples == 0 {
            return Err(Problem::new("never_runs", "this schedule never runs"));
        }
        Ok(())
    }

    /// The first scheduled time after `after`, Unix ms. `from` is the last
    /// scheduled time (or when the routine started), which intervals count
    /// from.
    pub fn next_after(&self, from: i64, after: i64) -> Option<i64> {
        match &self.kind {
            Kind::Interval { ms } => {
                let steps = if after < from {
                    1
                } else {
                    (after - from) / ms + 1
                };
                Some(from + steps * ms)
            }
            Kind::Weekly { days, time } => self.first(after, |date| {
                let weekday = date.weekday().to_monday_one_offset();
                (days & (1 << weekday) != 0).then(|| vec![*time])
            }),
            Kind::Cron(cron) => self.first(after, |date| {
                cron.matches_date(date).then(|| {
                    cron.times()
                        .filter_map(|(hour, minute)| Time::new(hour, minute, 0, 0).ok())
                        .collect()
                })
            }),
        }
    }

    /// The first local time on the days `times` returns that falls after
    /// `after`, walking the calendar from the day of `after`.
    fn first(&self, after: i64, times: impl Fn(Date) -> Option<Vec<Time>>) -> Option<i64> {
        let start = Timestamp::from_millisecond(after).ok()?;
        let mut date = self.tz.to_datetime(start).date();
        // The day before too: its late times can land after `after` once
        // a jump forward moves them.
        date = date.yesterday().ok()?;
        for _ in 0..SEARCH_DAYS {
            if let Some(times) = times(date) {
                for time in times {
                    let at = self.resolve(date.to_datetime(time))?;
                    if at > after {
                        return Some(at);
                    }
                }
            }
            date = date.tomorrow().ok()?;
        }
        None
    }

    /// The instant a wall-clock time stands for, by the rules above.
    fn resolve(&self, local: DateTime) -> Option<i64> {
        let instant = match self.tz.to_ambiguous_timestamp(local).offset() {
            AmbiguousOffset::Unambiguous { offset } => offset.to_timestamp(local).ok()?,
            AmbiguousOffset::Fold { before, .. } => before.to_timestamp(local).ok()?,
            AmbiguousOffset::Gap { before, after } => self.jump(local, before, after)?,
        };
        Some(instant.as_millisecond())
    }

    /// The instant clocks jumped over `local`: the first moment on the new
    /// offset, found between reading `local` with either offset.
    fn jump(&self, local: DateTime, before: Offset, after: Offset) -> Option<Timestamp> {
        let mut low = after.to_timestamp(local).ok()?.as_second();
        let mut high = before.to_timestamp(local).ok()?.as_second();
        while low < high {
            let mid = low + (high - low) / 2;
            let offset = self.tz.to_offset(Timestamp::from_second(mid).ok()?);
            if offset == after {
                high = mid;
            } else {
                low = mid + 1;
            }
        }
        Timestamp::from_second(high).ok()
    }

    /// `at` as the routine's zone shows it, like `2026-10-01 09:00`.
    pub fn local_time(&self, at: i64) -> String {
        Timestamp::from_millisecond(at)
            .map(|ts| {
                ts.to_zoned(self.tz.clone())
                    .strftime("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default()
    }
}

/// `HH:MM`, 24-hour.
fn clock_time(text: &str) -> Result<Time, Problem> {
    let bad = || {
        Problem::new(
            "time_invalid",
            format!("\"{text}\" is not a time like 09:00"),
        )
    };
    let (hour, minute) = text.split_once(':').ok_or_else(bad)?;
    if hour.len() != 2 || minute.len() != 2 {
        return Err(bad());
    }
    let hour: i8 = hour.parse().map_err(|_| bad())?;
    let minute: i8 = minute.parse().map_err(|_| bad())?;
    Time::new(hour, minute, 0, 0).map_err(|_| bad())
}

/// Unix ms of a wall-clock time in a zone, for tests and callers that
/// think in local time.
pub fn at(timezone: &str, year: i16, month: i8, day: i8, hour: i8, minute: i8) -> i64 {
    let tz = TimeZone::get(timezone).expect("known zone");
    jiff::civil::date(year, month, day)
        .at(hour, minute, 0, 0)
        .to_zoned(tz)
        .expect("valid time")
        .timestamp()
        .as_millisecond()
}

#[cfg(test)]
mod tests;
