//! Tests for [`super::Plan`].

use super::*;

const MINUTE: i64 = 60_000;
const NY: &str = "America/New_York";

fn weekly(days: &[u8], time: &str, tz: &str) -> Plan {
    Plan::new(
        &Schedule::Weekly {
            days: days.to_vec(),
            time: time.into(),
        },
        tz,
    )
    .expect("valid")
}

#[test]
fn weekdays_at_nine_in_the_routines_zone() {
    let plan = weekly(&[1, 2, 3, 4, 5], "09:00", "America/Sao_Paulo");
    // Friday 2026-10-02 at 10:00: next is Monday the 5th at 09:00.
    let now = at("America/Sao_Paulo", 2026, 10, 2, 10, 0);
    let next = plan.next_after(now, now).expect("next");
    assert_eq!(next, at("America/Sao_Paulo", 2026, 10, 5, 9, 0));
    assert_eq!(plan.local_time(next), "2026-10-05 09:00");
    // Exactly at the time: that one is past, the next day comes.
    assert_eq!(
        plan.next_after(next, next),
        Some(at("America/Sao_Paulo", 2026, 10, 6, 9, 0))
    );
}

#[test]
fn intervals_count_from_the_last_scheduled_time() {
    let plan = Plan::new(&Schedule::Interval { minutes: 120 }, "UTC").expect("valid");
    assert_eq!(plan.next_after(0, 0), Some(120 * MINUTE));
    // Late by 5 h: the next time keeps the 2 h grid, it does not drift.
    assert_eq!(plan.next_after(0, 300 * MINUTE), Some(360 * MINUTE));
}

#[test]
fn a_time_the_clock_skips_runs_right_after_the_jump() {
    // New York jumped from 02:00 to 03:00 on 2026-03-08.
    let plan = weekly(&[7], "02:30", NY);
    let before = at(NY, 2026, 3, 7, 12, 0);
    let next = plan.next_after(before, before).expect("next");
    assert_eq!(next, at(NY, 2026, 3, 8, 3, 0));
    assert_eq!(plan.local_time(next), "2026-03-08 03:00");
}

#[test]
fn a_time_that_happens_twice_runs_once() {
    // New York went back from 02:00 to 01:00 on 2026-11-01.
    let plan = weekly(&[7], "01:30", NY);
    let before = at(NY, 2026, 10, 31, 12, 0);
    let first = plan.next_after(before, before).expect("first");
    let second = plan.next_after(first, first).expect("second");
    assert_eq!(
        second,
        at(NY, 2026, 11, 8, 1, 30),
        "the next is a week later"
    );
    // The first 01:30 is still on daylight time (UTC-4).
    assert_eq!(
        first,
        Timestamp::from_second(1_793_511_000)
            .expect("ts")
            .as_millisecond()
    );
}

#[test]
fn cron_follows_the_zone_and_its_days() {
    let plan = Plan::new(
        &Schedule::Cron {
            expr: "30 8 * * mon".into(),
        },
        "Europe/Lisbon",
    )
    .expect("valid");
    let now = at("Europe/Lisbon", 2026, 10, 1, 0, 0);
    assert_eq!(
        plan.next_after(now, now),
        Some(at("Europe/Lisbon", 2026, 10, 5, 8, 30))
    );
    let leap = Plan::new(
        &Schedule::Cron {
            expr: "0 0 29 2 *".into(),
        },
        "UTC",
    )
    .expect("valid");
    let now = at("UTC", 2026, 10, 1, 0, 0);
    assert_eq!(
        leap.next_after(now, now),
        Some(at("UTC", 2028, 2, 29, 0, 0))
    );
}

#[test]
fn schedules_are_checked() {
    let now = at("UTC", 2026, 10, 1, 0, 0);
    let refused = [
        (
            Schedule::Weekly {
                days: vec![],
                time: "09:00".into(),
            },
            "UTC",
        ),
        (
            Schedule::Weekly {
                days: vec![8],
                time: "09:00".into(),
            },
            "UTC",
        ),
        (
            Schedule::Weekly {
                days: vec![1],
                time: "9:00".into(),
            },
            "UTC",
        ),
        (
            Schedule::Weekly {
                days: vec![1],
                time: "24:00".into(),
            },
            "UTC",
        ),
        (Schedule::Interval { minutes: 4 }, "UTC"),
        (Schedule::Interval { minutes: 10_081 }, "UTC"),
        (
            Schedule::Cron {
                expr: "* * * *".into(),
            },
            "UTC",
        ),
        (
            Schedule::Weekly {
                days: vec![1],
                time: "09:00".into(),
            },
            "Mars/Olympus",
        ),
    ];
    for (schedule, tz) in refused {
        assert!(Plan::new(&schedule, tz).is_err(), "{schedule:?} in {tz}");
    }
    let every_minute = Plan::new(
        &Schedule::Cron {
            expr: "* * * * *".into(),
        },
        "UTC",
    )
    .expect("parses");
    assert!(every_minute.check_spacing(now).is_err());
    let never = Plan::new(
        &Schedule::Cron {
            expr: "0 0 31 2 *".into(),
        },
        "UTC",
    )
    .expect("parses");
    assert!(never.check_spacing(now).is_err());
    let fine = Plan::new(
        &Schedule::Cron {
            expr: "*/5 * * * *".into(),
        },
        "UTC",
    )
    .expect("parses");
    assert!(fine.check_spacing(now).is_ok());
}
