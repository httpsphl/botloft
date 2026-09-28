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

/** "912 B", "48 KB", "3.4 MB". */
export function fileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  const kb = bytes / 1024;
  if (kb < 1024) {
    return `${Math.round(kb)} KB`;
  }
  const mb = kb / 1024;
  return `${mb < 10 ? mb.toFixed(1) : Math.round(mb)} MB`;
}

/** "0.8 s", "42 s", "3 min 5 s". */
export function duration(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 10) {
    return `${seconds.toFixed(1)} s`;
  }
  if (seconds < 60) {
    return `${Math.round(seconds)} s`;
  }
  const minutes = Math.floor(seconds / 60);
  const rest = Math.round(seconds % 60);
  return rest === 0 ? `${minutes} min` : `${minutes} min ${rest} s`;
}

/** "Monday, September 28" for today's year, with the year otherwise. */
export function day(ms: number, now = Date.now()): string {
  const sameYear = new Date(ms).getFullYear() === new Date(now).getFullYear();
  return new Intl.DateTimeFormat(undefined, {
    weekday: "long",
    month: "long",
    day: "numeric",
    year: sameYear ? undefined : "numeric",
  }).format(ms);
}
