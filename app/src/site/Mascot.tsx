// A mascot on the site (spec 39.4). The built page holds only its place, an
// empty drawing of its size (a whole mascot is too heavy to repeat in every
// page); the page's script puts a live one there, with the color, size and
// mood written on the place (main.tsx).

import type { Mood } from "../features/bots/BotAvatar";

export interface MascotProps {
  color: string;
  /** Its height in pixels; the mascot's drawing is as wide as it is tall. */
  size: number;
  mood: Mood;
  className?: string;
}

export function Mascot({ color, size, mood, className = "" }: MascotProps) {
  return (
    <span
      className={`site-mascot ${className}`}
      data-color={color}
      data-size={size}
      data-mood={mood}
      aria-hidden
    >
      <svg width={size} height={size} viewBox="0 0 1 1" />
    </span>
  );
}

/** What a place asks for, read back from the built page. */
export function readMascot(element: HTMLElement): MascotProps | null {
  const { color, size, mood } = element.dataset;
  const pixels = Number(size);
  if (!color || !mood || !Number.isFinite(pixels) || pixels <= 0) {
    return null;
  }
  return { color, size: pixels, mood: mood as Mood };
}
