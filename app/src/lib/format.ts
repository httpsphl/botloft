// Times as the owner reads them: clock time in the system's locale, and
// short relative phrases ("in 5 min") in English, like the rest of the UI.

const clock = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const dated = new Intl.DateTimeFormat(undefined, {
  month: "short",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
});
const relative = new Intl.RelativeTimeFormat("en", { numeric: "auto", style: "short" });

/** Clock time for today, date and time before that. */
export function when(ms: number, now = Date.now()): string {
  const same = new Date(ms).toDateString() === new Date(now).toDateString();
  return (same ? clock : dated).format(ms);
}

/** "in 5 min", "2 hr ago", "now". */
export function fromNow(ms: number, now = Date.now()): string {
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
