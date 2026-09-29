// Re-tints the mascot artwork (mascotArt.ts) to a bot's color. Every color
// of the drawing moves by the same step from the color it was drawn in to
// the bot's: the hue turns, the saturation scales and the lightness shifts,
// so light and shadow stay where they were drawn.

import { DRAWN_IN } from "./mascotArt";

/** `#RRGGBB` as hue (degrees), saturation and lightness (0 to 1). */
export function toHsl(hex: string): [number, number, number] {
  const value = Number.parseInt(hex.replace("#", "").slice(0, 6), 16);
  const r = ((value >> 16) & 255) / 255;
  const g = ((value >> 8) & 255) / 255;
  const b = (value & 255) / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  const chroma = max - min;
  if (chroma === 0) {
    return [0, 0, l];
  }
  const s = chroma / (1 - Math.abs(2 * l - 1));
  const sector =
    max === r ? ((g - b) / chroma) % 6 : max === g ? (b - r) / chroma + 2 : (r - g) / chroma + 4;
  return [(sector * 60 + 360) % 360, s, l];
}

/** Hue (degrees), saturation and lightness (0 to 1) as `#rrggbb`. */
export function toHex(h: number, s: number, l: number): string {
  const chroma = (1 - Math.abs(2 * l - 1)) * s;
  const x = chroma * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = l - chroma / 2;
  const [r, g, b] =
    h < 60
      ? [chroma, x, 0]
      : h < 120
        ? [x, chroma, 0]
        : h < 180
          ? [0, chroma, x]
          : h < 240
            ? [0, x, chroma]
            : h < 300
              ? [x, 0, chroma]
              : [chroma, 0, x];
  const byte = (channel: number) =>
    Math.round((channel + m) * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${byte(r)}${byte(g)}${byte(b)}`;
}

const unit = (value: number) => Math.max(0, Math.min(1, value));

/** A color of the artwork, `drawn`, moved to the bot's `color`. */
export function retint(drawn: string, color: string): string {
  const [h, s, l] = toHsl(drawn);
  const [hue, saturation, lightness] = toHsl(color);
  return toHex(
    (h + hue - DRAWN_IN.h + 360) % 360,
    unit(s * Math.min(1.1, saturation / DRAWN_IN.s)),
    unit(l + (lightness - DRAWN_IN.l) * 0.9),
  );
}
