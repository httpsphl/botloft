// The owner's operating system, by the name texts give it (spec 15.6): the
// settings say "Start with Windows" on Windows and "Start with macOS" on a
// Mac. Read from the app window's user agent; anything unknown is Windows,
// where Botloft started.

export type SystemName = "Windows" | "macOS" | "Linux";

export function systemName(
  agent: string = typeof navigator === "undefined" ? "" : navigator.userAgent,
): SystemName {
  if (/Macintosh|Mac OS X|darwin/i.test(agent)) {
    return "macOS";
  }
  if (/Linux|X11/i.test(agent) && !/Android/i.test(agent)) {
    return "Linux";
  }
  return "Windows";
}

/** This computer's system, read once. */
export const SYSTEM: SystemName = systemName();
