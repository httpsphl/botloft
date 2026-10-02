// Conversions for the color picker: `#RRGGBB`, red/green/blue channels
// (0 to 255) and hue/saturation/value (hue in degrees, the rest 0 to 1).
// The picker keeps the color as HSV, so dragging to gray or black does not
// lose the hue the owner chose.

export type Rgb = [number, number, number];
export type Hsv = { h: number; s: number; v: number };

/** `#RRGGBB` (any case, `#` optional) as channels, or null if it is not one. */
export function parseHex(input: string): Rgb | null {
  const hex = input.trim().replace(/^#/, "");
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) {
    return null;
  }
  const value = Number.parseInt(hex, 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

/** Channels as `#RRGGBB`, uppercased like the daemon stores it. */
export function toHexColor([r, g, b]: Rgb): string {
  const byte = (channel: number) =>
    Math.round(Math.max(0, Math.min(255, channel)))
      .toString(16)
      .padStart(2, "0");
  return `#${byte(r)}${byte(g)}${byte(b)}`.toUpperCase();
}

export function rgbToHsv([r, g, b]: Rgb): Hsv {
  const [red, green, blue] = [r / 255, g / 255, b / 255];
  const max = Math.max(red, green, blue);
  const chroma = max - Math.min(red, green, blue);
  let h = 0;
  if (chroma > 0) {
    const sector =
      max === red
        ? ((green - blue) / chroma) % 6
        : max === green
          ? (blue - red) / chroma + 2
          : (red - green) / chroma + 4;
    h = (sector * 60 + 360) % 360;
  }
  return { h, s: max === 0 ? 0 : chroma / max, v: max };
}

export function hsvToRgb({ h, s, v }: Hsv): Rgb {
  const at = (n: number) => {
    const k = (n + h / 60) % 6;
    return Math.round((v - v * s * Math.max(0, Math.min(k, 4 - k, 1))) * 255);
  };
  return [at(5), at(3), at(1)];
}
