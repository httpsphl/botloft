// Which download the site offers first (spec 39.3): the visitor's own
// system, read from what the browser says about itself. The links go to
// the account server's `/download/<system>`, which knows the newest version.

export type System = "windows" | "macos" | "linux" | "phone";

/** The address of each installer, on this site. */
export const DOWNLOADS = {
  windows: "/download/windows",
  macos: "/download/macos",
  deb: "/download/linux-deb",
  appImage: "/download/linux-appimage",
} as const;

/** The download the main button gives each system; a phone goes to the list. */
export const MAIN_DOWNLOAD: Record<System, string> = {
  windows: DOWNLOADS.windows,
  macos: DOWNLOADS.macos,
  linux: DOWNLOADS.deb,
  phone: "#downloads",
};

/**
 * The system from the browser's own words: `platform` is
 * `navigator.userAgentData.platform` or `navigator.platform`, when there is
 * one, and `touches` is `navigator.maxTouchPoints`. Anything unknown is
 * Windows, the system Botloft knows best.
 */
export function systemOf(userAgent: string, platform = "", touches = 0): System {
  const said = `${platform} ${userAgent}`.toLowerCase();
  if (/android|iphone|ipad|ipod|mobile/.test(said)) return "phone";
  // An iPad asking for the desktop site says it is a Mac, but has a touch screen.
  if (/mac/.test(said)) return touches > 1 ? "phone" : "macos";
  if (/linux|x11|cros/.test(said)) return "linux";
  return "windows";
}
