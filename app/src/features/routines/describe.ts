// How a routine's times read (spec 20.9): the schedule in plain words
// ("Weekdays at 09:00"), the next time ("tomorrow at 09:00") and the time
// zones by city. Day and time words come from Intl, in the app's language.

import { currentLocale, type Messages } from "../../i18n";
import type { Schedule } from "../../lib/protocol.gen";

type Words = Messages["routines"];

/** A weekday's name; 1 = Monday … 7 = Sunday. */
export function weekdayName(day: number, width: "long" | "short" | "narrow" = "long"): string {
  // 1 January 2024 was a Monday.
  const date = new Date(Date.UTC(2024, 0, day));
  return new Intl.DateTimeFormat(currentLocale(), { weekday: width, timeZone: "UTC" }).format(date);
}

/** `HH:MM` as the app's language writes a time of day. */
export function clockText(time: string): string {
  const [hours = 0, minutes = 0] = time.split(":").map(Number);
  const date = new Date(Date.UTC(2024, 0, 1, hours, minutes));
  return new Intl.DateTimeFormat(currentLocale(), {
    hour: "2-digit",
    minute: "2-digit",
    timeZone: "UTC",
  }).format(date);
}

const capitalize = (text: string) => text.charAt(0).toLocaleUpperCase() + text.slice(1);

export function describeSchedule(schedule: Schedule, words: Words["when"]): string {
  switch (schedule.kind) {
    case "weekly": {
      const days = [...new Set(schedule.days)].sort((a, b) => a - b);
      const time = clockText(schedule.time);
      switch (days.join(",")) {
        case "1,2,3,4,5,6,7":
          return words.everyDay(time);
        case "1,2,3,4,5":
          return words.weekdays(time);
        case "6,7":
          return words.weekends(time);
        default: {
          const list = new Intl.ListFormat(currentLocale(), { type: "conjunction" });
          return words.days(capitalize(list.format(days.map((day) => weekdayName(day)))), time);
        }
      }
    }
    case "interval":
      return schedule.minutes % 60 === 0
        ? words.everyHours(schedule.minutes / 60)
        : words.everyMinutes(schedule.minutes);
    case "cron":
      return words.cron(schedule.expr);
    case "signal":
      return words.signal(schedule.name);
  }
}

/** The calendar day of `ms` in `timeZone`, to compare days. */
function dayKey(ms: number, timeZone: string): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(ms);
}

/** "today at 09:00", "tomorrow at 09:00" or "Mon, Oct 5 at 09:00", in the routine's zone. */
export function nextText(ms: number, timeZone: string, words: Words, now = Date.now()): string {
  const locale = currentLocale();
  const zone = knownZone(timeZone);
  const time = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    timeZone: zone,
  }).format(ms);
  const day = dayKey(ms, zone);
  if (day === dayKey(now, zone)) {
    return words.today(time);
  }
  if (day === dayKey(now + 86_400_000, zone)) {
    return words.tomorrow(time);
  }
  const date = new Intl.DateTimeFormat(locale, {
    weekday: "short",
    day: "numeric",
    month: "short",
    timeZone: zone,
  }).format(ms);
  return words.onDay(date, time);
}

/** The zone Windows is set to. */
export function systemZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
}

/** Every zone the webview knows, the system's and UTC included. */
export function allZones(): string[] {
  const listed =
    typeof Intl.supportedValuesOf === "function" ? Intl.supportedValuesOf("timeZone") : [];
  return [...new Set([systemZone(), "UTC", ...listed])].sort();
}

/** "America/Sao_Paulo" -> "Sao Paulo (America)". */
export function zoneLabel(zone: string): string {
  const parts = zone.split("/");
  const city = (parts.at(-1) ?? zone).replaceAll("_", " ");
  return parts.length > 1 ? `${city} (${parts[0]})` : city;
}

function knownZone(zone: string): string {
  try {
    new Intl.DateTimeFormat("en", { timeZone: zone });
    return zone;
  } catch {
    return "UTC";
  }
}
