// The fake daemon's rules for a browser's pages, close enough to the
// daemon's for the UI: the size of a page in a panel (spec 21.3) and the
// addresses the owner may type (spec 21.10).

/** The page's size: as wide as ever, as tall as the panel's room asks. */
export interface PageSize {
  width: number;
  height: number;
}

export const PAGE: PageSize = { width: 1280, height: 800 };

/** The page for a panel with `width` by `height` of room, as the daemon fits it. */
export function fitting(width: number, height: number): PageSize {
  const wide = Math.min(1600, Math.max(800, width));
  const tall = Math.floor((wide * height) / Math.max(1, width));
  return { width: wide, height: Math.min(2000, Math.max(600, tall)) };
}

/**
 * What the owner typed, as the web address to open: `https://` goes in
 * front of one without a scheme. `null` for anything else.
 */
export function typed(input: string): string | null {
  const text = input.trim();
  if (!text || /\s/.test(text)) {
    return null;
  }
  const web = /^https?:\/\//i.test(text);
  // `mailto:x` has a scheme; `localhost:3000` has a port.
  if (!web && /^[a-z][a-z0-9+.-]+:(?!\d+([/?#]|$))/i.test(text)) {
    return null;
  }
  const url = web ? text : `https://${text}`;
  try {
    return new URL(url).hostname ? url : null;
  } catch {
    return null;
  }
}
