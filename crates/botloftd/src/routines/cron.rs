//! 5-field cron expressions (spec 20.2): minute, hour, day of month, month
//! and day of week, with `*`, lists, ranges, steps and English names. As in
//! classic cron, when both day fields are restricted a day matches either.

use jiff::civil::Date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cron {
    minutes: u64,
    hours: u64,
    days: u64,
    months: u64,
    /// Bit 0 is Sunday; 7 is folded into it.
    weekdays: u64,
    /// Day of month and day of week each start with `*`.
    any_day: bool,
    any_weekday: bool,
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];
const WEEKDAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];

impl Cron {
    pub fn parse(expr: &str) -> Result<Self, String> {
        let fields: Vec<&str> = expr.split_whitespace().collect();
        let [minute, hour, day, month, weekday] = fields[..] else {
            return Err("a cron expression has 5 fields: minute hour day month weekday".into());
        };
        let mut weekdays = field(weekday, 0, 7, &WEEKDAYS, 0, "day of week")?;
        if weekdays & (1 << 7) != 0 {
            weekdays = (weekdays & !(1 << 7)) | 1;
        }
        Ok(Self {
            minutes: field(minute, 0, 59, &[], 0, "minute")?,
            hours: field(hour, 0, 23, &[], 0, "hour")?,
            days: field(day, 1, 31, &[], 0, "day of month")?,
            months: field(month, 1, 12, &MONTHS, 1, "month")?,
            weekdays,
            any_day: day.starts_with('*'),
            any_weekday: weekday.starts_with('*'),
        })
    }

    /// Whether the expression runs on `date` at all.
    pub fn matches_date(&self, date: Date) -> bool {
        let month = u32::try_from(date.month()).unwrap_or(0);
        if self.months & (1 << month) == 0 {
            return false;
        }
        let day = self.days & (1 << u32::try_from(date.day()).unwrap_or(0)) != 0;
        let weekday = self.weekdays
            & (1 << u32::try_from(date.weekday().to_sunday_zero_offset()).unwrap_or(0))
            != 0;
        match (self.any_day, self.any_weekday) {
            (false, false) => day || weekday,
            _ => day && weekday,
        }
    }

    /// The times of day it runs, in order.
    pub fn times(&self) -> impl Iterator<Item = (i8, i8)> + '_ {
        (0..24i8)
            .filter(|h| self.hours & (1 << h) != 0)
            .flat_map(move |hour| {
                (0..60i8)
                    .filter(move |m| self.minutes & (1 << m) != 0)
                    .map(move |minute| (hour, minute))
            })
    }
}

/// One field as a bit set of the values it allows.
fn field(
    text: &str,
    min: u32,
    max: u32,
    names: &[&str],
    first_name: u32,
    what: &str,
) -> Result<u64, String> {
    let bad = || format!("the {what} field \"{text}\" is not valid");
    let value = |part: &str| -> Result<u32, String> {
        let lower = part.to_ascii_lowercase();
        if let Some(index) = names.iter().position(|name| *name == lower) {
            return Ok(first_name + u32::try_from(index).unwrap_or(0));
        }
        let number: u32 = part.parse().map_err(|_| bad())?;
        if (min..=max).contains(&number) {
            Ok(number)
        } else {
            Err(format!("the {what} field allows {min} to {max}"))
        }
    };
    let mut set = 0u64;
    for part in text.split(',') {
        let (range, step) = match part.split_once('/') {
            Some((range, step)) => {
                let step: u32 = step.parse().map_err(|_| bad())?;
                if step == 0 {
                    return Err(bad());
                }
                (range, step)
            }
            None => (part, 1),
        };
        let (from, to) = if range == "*" {
            (min, max)
        } else if let Some((from, to)) = range.split_once('-') {
            (value(from)?, value(to)?)
        } else {
            let from = value(range)?;
            // `5/15` runs from 5 to the end, like `5-59/15`.
            (from, if part.contains('/') { max } else { from })
        };
        if from > to {
            return Err(bad());
        }
        let mut at = from;
        while at <= to {
            set |= 1 << at;
            at += step;
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    #[test]
    fn fields_take_lists_ranges_steps_and_names() {
        let cron = Cron::parse("*/15 9-17 * * mon-fri").expect("valid");
        let times: Vec<_> = cron.times().take(5).collect();
        assert_eq!(times, [(9, 0), (9, 15), (9, 30), (9, 45), (10, 0)]);
        // 2026-10-02 is a Friday, 2026-10-03 a Saturday.
        assert!(cron.matches_date(date(2026, 10, 2)));
        assert!(!cron.matches_date(date(2026, 10, 3)));
        assert!(
            Cron::parse("0 0 1 JAN,jul *")
                .expect("names")
                .matches_date(date(2026, 7, 1))
        );
        assert!(
            Cron::parse("0 12 * * 7")
                .expect("sunday")
                .matches_date(date(2026, 10, 4))
        );
    }

    #[test]
    fn both_day_fields_restricted_match_either() {
        // The 13th, or any Friday.
        let cron = Cron::parse("0 0 13 * 5").expect("valid");
        assert!(cron.matches_date(date(2026, 10, 13)));
        assert!(cron.matches_date(date(2026, 10, 2)));
        assert!(!cron.matches_date(date(2026, 10, 3)));
        // A `*` day of month makes the weekday decide alone.
        let fridays = Cron::parse("0 0 * * 5").expect("valid");
        assert!(!fridays.matches_date(date(2026, 10, 13)));
    }

    #[test]
    fn bad_expressions_say_what_is_wrong() {
        for (expr, part) in [
            ("* * * *", "5 fields"),
            ("60 * * * *", "minute"),
            ("* 24 * * *", "hour"),
            ("* * 0 * *", "day of month"),
            ("* * * 13 *", "month"),
            ("* * * * 8", "day of week"),
            ("*/0 * * * *", "minute"),
            ("5-1 * * * *", "minute"),
        ] {
            let err = Cron::parse(expr).expect_err(expr);
            assert!(err.contains(part), "{expr}: {err}");
        }
    }
}
