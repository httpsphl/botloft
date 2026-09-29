// Where the bot is writing on a screen (spec 22.3): what the script in each
// draft tells the app. The page is the bot's, so only numbers are read, and
// they are kept inside the page.

export interface Mark {
  /** The end of what is written, in the page's pixels. */
  point: { x: number; y: number } | null;
  /** The part of the page around it. */
  box: { x: number; y: number; width: number; height: number } | null;
}

const finite = (value: unknown): value is number =>
  typeof value === "number" && Number.isFinite(value);
const clamp = (value: number, max: number) => Math.min(max, Math.max(0, value));

/**
 * The mark in a message, in the pixels of `page`; `undefined` when the
 * message is not one. The script measures in the view it sees, which is
 * wider than the page where an outer zoom scales the frame (the preview's).
 */
export function readMark(data: unknown, page: { width: number; height: number }): Mark | undefined {
  if (typeof data !== "object" || data === null) {
    return undefined;
  }
  const { botloft, view, point, box } = data as {
    botloft?: unknown;
    view?: { width?: unknown; height?: unknown } | null;
    point?: { x?: unknown; y?: unknown } | null;
    box?: { x?: unknown; y?: unknown; width?: unknown; height?: unknown } | null;
  };
  if (botloft !== "cursor" || !view || !finite(view.width) || view.width < 1) {
    return undefined;
  }
  const factor = page.width / view.width;
  let found: Mark["box"] = null;
  if (box && finite(box.x) && finite(box.y) && finite(box.width) && finite(box.height)) {
    const left = clamp(box.x * factor, page.width);
    const top = clamp(box.y * factor, page.height);
    const right = clamp((box.x + box.width) * factor, page.width);
    const bottom = clamp((box.y + box.height) * factor, page.height);
    if (right - left >= 1 && bottom - top >= 1) {
      found = { x: left, y: top, width: right - left, height: bottom - top };
    }
  }
  return {
    point:
      point && finite(point.x) && finite(point.y)
        ? { x: clamp(point.x * factor, page.width), y: clamp(point.y * factor, page.height) }
        : null,
    box: found,
  };
}
