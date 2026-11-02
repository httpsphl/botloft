// The tones the app colors things by (spec 15.3), each with the classes
// for its uses, written out whole so Tailwind finds them. One table, so a
// badge, a card's icon and a notice of the same tone always match.

export type Tone = "accent" | "work" | "ok" | "warn" | "danger" | "quiet";

interface ToneClasses {
  /** An icon or a mark in the tone. */
  text: string;
  /** A soft fill with the tone on it: badges, a card's round icon. Words
   *  in the accent use `accent-text` (spec 15.3). */
  soft: string;
  /** A notice's frame: border and a faint fill. */
  frame: string;
  /** A solid fill, for counts. */
  solid: string;
}

export const TONES: Record<Tone, ToneClasses> = {
  accent: {
    text: "text-accent",
    soft: "bg-accent/12 text-accent-text",
    frame: "border-accent/40 bg-accent/6",
    solid: "bg-accent text-canvas",
  },
  work: {
    text: "text-work",
    soft: "bg-work/12 text-work",
    frame: "border-work/40 bg-work/6",
    solid: "bg-work text-canvas",
  },
  ok: {
    text: "text-ok",
    soft: "bg-ok/12 text-ok",
    frame: "border-ok/40 bg-ok/6",
    solid: "bg-ok text-canvas",
  },
  warn: {
    text: "text-warn",
    soft: "bg-warn/12 text-warn",
    frame: "border-warn/40 bg-warn/6",
    solid: "bg-warn text-canvas",
  },
  danger: {
    text: "text-danger",
    soft: "bg-danger/12 text-danger",
    frame: "border-danger/40 bg-danger/6",
    solid: "bg-danger text-canvas",
  },
  quiet: {
    text: "text-quiet",
    soft: "bg-sunken text-muted",
    frame: "border-line bg-panel",
    solid: "bg-quiet text-canvas",
  },
};
