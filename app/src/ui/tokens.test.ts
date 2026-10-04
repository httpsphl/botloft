// The color tokens (spec 15.3): every color words are written in reads at
// 4.5:1 or more (WCAG AA) on every background, in both themes.

import { describe, expect, test } from "vitest";
import css from "../index.css?raw";

/** The `--name: #hex` tokens of the first block that opens with `selector {`. */
function tokens(selector: string): Record<string, string> {
  const start = css.indexOf(`${selector} {`);
  const block = css.slice(start, css.indexOf("}", start));
  return Object.fromEntries(
    [...block.matchAll(/--([\w-]+):\s*(#[0-9a-f]{6})\b/gi)].map((m) => [m[1], m[2]]),
  );
}

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((at) => {
    const c = Number.parseInt(hex.slice(at, at + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)].sort((p, q) => q - p) as [number, number];
  return (x + 0.05) / (y + 0.05);
}

const BACKGROUNDS = ["canvas", "panel", "sunken"];
/** Colors text is written in. `accent` is a fill; words use `accent-text`. */
const TEXT = [
  "ink",
  "ink-soft",
  "muted",
  "accent-text",
  "ok",
  "work",
  "warn",
  "danger",
  "idle-state",
  "mention",
];

describe.each([
  ["light", ":root"],
  ["dark", '[data-theme="dark"]'],
])("the %s theme", (_name, selector) => {
  const theme = tokens(selector);

  test.each(TEXT)("%s reads on every background", (text) => {
    for (const background of BACKGROUNDS) {
      const ratio = contrast(theme[text] as string, theme[background] as string);
      expect(ratio, `${text} on ${background}`).toBeGreaterThanOrEqual(4.5);
    }
  });
});
