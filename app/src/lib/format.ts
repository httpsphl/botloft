// Times as the owner reads them, in the language the app shows (spec
// 15.6): clock time, dates and short relative phrases ("in 5 min").

import { currentLocale, type Locale } from "../i18n";
import type { TokenUsage } from "./protocol.gen";

interface Formats {
  clock: Intl.DateTimeFormat;
  dated: Intl.DateTimeFormat;
  relative: Intl.RelativeTimeFormat;
  day: Intl.DateTimeFormat;
  dayOfYear: Intl.DateTimeFormat;
  /** By number of decimals. */
  decimals: [Intl.NumberFormat, Intl.NumberFormat];
  /** US dollars, the language's way ("$12.40", "US$ 12,40"). */
  dollars: Intl.NumberFormat;
  /** "Sat, Nov 14". */
  shortDay: Intl.DateTimeFormat;
}

const cache = new Map<Locale, Formats>();

/** The formatters for the current language, made once per language. */
function formats(): Formats {
  const locale = currentLocale();
  let found = cache.get(locale);
  if (!found) {
    found = {
      clock: new Intl.DateTimeFormat(locale, { hour: "2-digit", minute: "2-digit" }),
      dated: new Intl.DateTimeFormat(locale, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      }),
      relative: new Intl.RelativeTimeFormat(locale, { numeric: "auto", style: "short" }),
      day: new Intl.DateTimeFormat(locale, { weekday: "long", month: "long", day: "numeric" }),
      dayOfYear: new Intl.DateTimeFormat(locale, {
        weekday: "long",
        month: "long",
        day: "numeric",
        year: "numeric",
      }),
      decimals: [0, 1].map(
        (digits) =>
          new Intl.NumberFormat(locale, {
            minimumFractionDigits: digits,
            maximumFractionDigits: digits,
          }),
      ) as Formats["decimals"],
      dollars: new Intl.NumberFormat(locale, { style: "currency", currency: "USD" }),
      shortDay: new Intl.DateTimeFormat(locale, {
        weekday: "short",
        month: "short",
        day: "numeric",
      }),
    };
    cache.set(locale, found);
  }
  return found;
}

/** Clock time for today, date and time before that. */
export function when(ms: number, now = Date.now()): string {
  const same = new Date(ms).toDateString() === new Date(now).toDateString();
  const { clock, dated } = formats();
  return (same ? clock : dated).format(ms);
}

/** "in 5 min", "2 hr ago", "now". */
export function fromNow(ms: number, now = Date.now()): string {
  const { relative } = formats();
  const seconds = Math.round((ms - now) / 1000);
  const abs = Math.abs(seconds);
  if (abs < 45) {
    return relative.format(0, "second");
  }
  if (abs < 45 * 60) {
    return relative.format(Math.round(seconds / 60), "minute");
  }
  if (abs < 36 * 3600) {
    return relative.format(Math.round(seconds / 3600), "hour");
  }
  return relative.format(Math.round(seconds / 86_400), "day");
}

/** `value` with `digits` decimals, written the language's way ("3.4", "3,4"). */
function decimal(value: number, digits: 0 | 1): string {
  return formats().decimals[digits].format(value);
}

/** "912 B", "48 KB", "3.4 MB", "15.6 GB". */
export function fileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const kb = bytes / 1024;
  if (kb < 1024) {
    return `${Math.round(kb)} KB`;
  }
  const mb = kb / 1024;
  if (mb < 1024) {
    return `${mb < 10 ? decimal(mb, 1) : Math.round(mb)} MB`;
  }
  return `${decimal(mb / 1024, 1)} GB`;
}

/** Processors from hundredths: 200 is "2", 150 is "1.5" (spec 42.4). */
export function processors(percent: number): string {
  return decimal(percent / 100, percent % 100 === 0 ? 0 : 1);
}

/**
 * A cost at API prices in US dollars (spec 8.7): "$12.40" ("US$ 12,40" in
 * Portuguese); under a cent, "< $0.01".
 */
export function dollars(amount: number): string {
  const { dollars } = formats();
  return amount > 0 && amount < 0.005 ? `< ${dollars.format(0.01)}` : dollars.format(amount);
}

/** A day the daemon wrote as `2026-11-14`, as "Sat, Nov 14". */
export function shortDay(key: string): string {
  const [year, month, day] = key.split("-").map(Number);
  if (!year || !month || !day) {
    return key;
  }
  return formats().shortDay.format(new Date(year, month - 1, day));
}

/** Tokens as the owner reads them: "850", "24.3k", "556k", "1M", "1.2M". */
/** A share of the plan, 0 to 1, as a percentage: "4.5%", "<0.1%". */
export function planPercent(share: number): string {
  const percent = share * 100;
  if (percent <= 0) {
    return `${decimal(0, 0)}%`;
  }
  if (percent < 0.05) {
    return `<${decimal(0.1, 1)}%`;
  }
  return `${decimal(percent, percent < 10 ? 1 : 0)}%`;
}

export function tokens(count: number): string {
  if (count < 1000) {
    return String(count);
  }
  const thousands = count / 1000;
  if (thousands < 100) {
    return `${decimal(thousands, thousands % 1 < 0.05 ? 0 : 1)}k`;
  }
  if (thousands < 999.5) {
    return `${Math.round(thousands)}k`;
  }
  const millions = count / 1_000_000;
  return `${decimal(millions, Math.abs(millions - Math.round(millions)) < 0.05 ? 0 : 1)}M`;
}

/**
 * The tokens a turn counts for (spec 8.7): input read, with what went into
 * the prompt cache (the conversation sent again after a pause too), and
 * what the model wrote. The conversation reread from the cache is left out.
 */
export function usedTokens(usage: TokenUsage): number {
  return usage.input + usage.cacheWrite + usage.output;
}

/** "0.8 s", "42 s", "3 min 5 s". */
export function duration(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 10) {
    return `${decimal(seconds, 1)} s`;
  }
  if (seconds < 60) {
    return `${Math.round(seconds)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  const rest = Math.round(seconds % 60);
  return rest === 0 ? `${minutes} min` : `${minutes} min ${rest} s`;
}

/** Clock time only, whatever the day. */
export function clock(ms: number): string {
  return formats().clock.format(ms);
}

/** "Monday, September 28" for today's year, with the year otherwise. */
export function day(ms: number, now = Date.now()): string {
  const sameYear = new Date(ms).getFullYear() === new Date(now).getFullYear();
  const { day, dayOfYear } = formats();
  return (sameYear ? day : dayOfYear).format(ms);
}
