// The parts of a working mascot that move with its flame, step by step on
// the flame's timer (flame.ts): what the badge draws (MascotBadge.tsx) and
// the Maker's wand and flow (MakerGear.tsx). Each part says how it moves
// in `data-tick`; `at` is the time in seconds, already shifted by the
// mascot's own offset so a crew's mascots do not move together. A part
// that moves must not carry a transform of its own: wrap it instead.

type Step = (part: SVGElement, at: number) => void;

const round = (value: number) => Math.round(value * 10) / 10;

const STEPS: Record<string, Step> = {
  /** Turns round, as a radar's sweep. */
  spin: (part, at) => part.setAttribute("transform", `rotate(${round((at * 240) % 360)})`),
  /** Narrows and widens, as a globe's meridian turning. */
  turn: (part, at) => part.setAttribute("rx", String(round(6 + Math.abs(Math.cos(at * 2.4)) * 56))),
  /** Writes: a quick rock with a little stroke. */
  wiggle: (part, at) =>
    part.setAttribute(
      "transform",
      `translate(${round(Math.sin(at * 3) * 10)} 0) rotate(${round(Math.sin(at * 11) * 7)})`,
    ),
  /** On for half a second, off for half. */
  blink: (part, at) => part.setAttribute("opacity", at % 1 < 0.5 ? "1" : "0"),
  /** Wanders in a small loop, looking. */
  scan: (part, at) =>
    part.setAttribute(
      "transform",
      `translate(${round(Math.cos(at * 2) * 10)} ${round(Math.sin(at * 3) * 8)})`,
    ),
  /** Lights after the dot before it. */
  dot: (part, at) => {
    const phase = Number(part.dataset.phase ?? 0);
    const glow = Math.max(0, Math.sin(at * 5 - phase * 0.9));
    part.setAttribute("opacity", String(round(0.25 + 0.75 * glow)));
  },
  /** The Maker's wand, waving slowly about its grip. */
  wave: (part, at) => part.setAttribute("transform", `rotate(${round(Math.sin(at * 2.6) * 18)})`),
  /** A spark thrown off the wand's tip at each wave, rising as it fades. */
  fling: (part, at) => {
    const phase = Number(part.dataset.phase ?? 0);
    const flight = (at * 0.83 + phase / 2) % 1;
    part.setAttribute(
      "transform",
      `translate(${round(flight * 60 * (phase ? -1 : 1))} ${round(-flight * 140)}) scale(${round(1 - flight * 0.7)})`,
    );
    part.setAttribute("opacity", String(round(flight < 0.15 ? flight / 0.15 : 1 - flight)));
  },
  /** One box of the Maker's flow, lit in its turn along the arrows. */
  light: (part, at) => {
    const phase = Number(part.dataset.phase ?? 0);
    const count = Number(part.dataset.count ?? 1);
    const lit = Math.floor(at * 1.2) % count === phase;
    part.setAttribute("data-lit", lit ? "" : "off");
  },
  /** The dots of an arrow running toward its head. */
  run: (part, at) => part.setAttribute("stroke-dashoffset", String(round(-(at * 90) % 60))),
};

/** Steps every moving part of `svg` to time `at` (seconds). */
export function stepParts(svg: SVGSVGElement, at: number): void {
  for (const part of svg.querySelectorAll<SVGElement>("[data-tick]")) {
    STEPS[part.dataset.tick ?? ""]?.(part, at);
  }
}

/** What each kind of step sets, to take off again; a part keeps what React drew. */
const SETS: Record<string, string[]> = {
  spin: ["transform"],
  wiggle: ["transform"],
  scan: ["transform"],
  wave: ["transform"],
  fling: ["transform"],
  blink: ["opacity"],
  dot: ["opacity"],
  light: ["data-lit"],
  run: ["stroke-dashoffset"],
};

/** Puts every moving part of `svg` back as drawn. */
export function settleParts(svg: SVGSVGElement): void {
  for (const part of svg.querySelectorAll<SVGElement>("[data-tick]")) {
    const tick = part.dataset.tick ?? "";
    for (const attribute of SETS[tick] ?? []) part.removeAttribute(attribute);
    if (tick === "fling") part.setAttribute("opacity", "0");
    if (tick === "turn" && part.dataset.rx) part.setAttribute("rx", part.dataset.rx);
  }
}
