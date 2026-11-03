// Small gestures an awake mascot makes now and then (spec 15.3): a little
// bob, a stretch up, a sway. Every few seconds one awake or waiting
// mascot on screen may make one, picked by weight from its own seeded
// draw and never the one it made last, so each mascot keeps its own rhythm
// and a crew does not move together. Played with GSAP as the moments are
// (gestures.ts); none while the mascots stand still (eyeMoments.ts).

import { resting } from "./eyeMoments";
import { AT_FEET, body, type Parts, play } from "./gestures";

export type Fidget = "bob" | "stretch" | "sway";

/** How often the scheduler looks, and how often each mascot fidgets on average. */
const TICK_MS = 2000;
const EVERY_MS = 16000;

const WEIGHTS: [Fidget, number][] = [
  ["bob", 3],
  ["stretch", 2],
  ["sway", 2],
];

const GESTURES: Record<Fidget, (tl: gsap.core.Timeline, parts: Parts) => void> = {
  bob: (tl, { spring }) =>
    body(tl, spring, [
      [0.1, [1.06, 0.94], "power1.out"],
      [0.26, [0.97, 1.04, -40], "power2.out"],
      [0.4, [1.05, 0.95], "power2.in"],
      [0.54, [0.99, 1.01, -16], "power1.out"],
      [0.68, [1.02, 0.98], "power1.in"],
      [0.9, [1, 1], "sine.out"],
    ]),
  stretch: (tl, { spring }) =>
    body(tl, spring, [
      [0.35, [0.95, 1.09], "sine.out"],
      [0.85, [0.95, 1.1], "none"],
      [1.05, [1.04, 0.96], "power2.in"],
      [1.3, [1, 1], "back.out(2)"],
    ]),
  sway: (tl, { spring }) => {
    if (!spring) return;
    tl.to(spring, { rotation: -5, ...AT_FEET, duration: 0.35, ease: "sine.inOut" })
      .to(spring, { rotation: 5, ...AT_FEET, duration: 0.5, ease: "sine.inOut" })
      .to(spring, { rotation: 0, ...AT_FEET, duration: 0.45, ease: "back.out(2)" });
  },
};

/** A small fast seeded random, from a string. */
export function seeded(seed: string): () => number {
  let state = 0;
  for (const char of seed) state = (Math.imul(state, 31) + char.charCodeAt(0)) | 0;
  return () => {
    state = (state + 0x6d2b79f5) | 0;
    let mixed = Math.imul(state ^ (state >>> 15), 1 | state);
    mixed = (mixed + Math.imul(mixed ^ (mixed >>> 7), 61 | mixed)) ^ mixed;
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
  };
}

interface Draw {
  random: () => number;
  last: Fidget | null;
}

const draws = new WeakMap<SVGSVGElement, Draw>();

/** The next fidget from `draw`: by weight, never the one before. */
export function nextFidget(draw: Draw): Fidget {
  const choices = WEIGHTS.filter(([fidget]) => fidget !== draw.last);
  const total = choices.reduce((sum, [, weight]) => sum + weight, 0);
  let pick = draw.random() * total;
  for (const [fidget, weight] of choices) {
    pick -= weight;
    if (pick < 0) {
      draw.last = fidget;
      return fidget;
    }
  }
  const fallback = choices[0]?.[0] ?? "bob";
  draw.last = fallback;
  return fallback;
}

/** `svg` makes `fidget`. */
export function fidget(svg: SVGSVGElement, kind: Fidget): void {
  play(svg, GESTURES[kind]);
}

/** Awake or waiting mascots on screen that are not in a gesture already. */
function candidates(): SVGSVGElement[] {
  const all = document.querySelectorAll<SVGSVGElement>(
    'svg[data-mood="awake"]:not([data-still], [data-gesture]), svg[data-mood="waiting"]:not([data-still], [data-gesture])',
  );
  return [...all].filter((svg) => {
    const box = svg.getBoundingClientRect();
    return box.width > 0 && box.bottom > 0 && box.top < innerHeight;
  });
}

/** Starts the fidgets; returns what stops them. */
export function startFidgets(random: () => number = Math.random): () => void {
  const timer = setInterval(() => {
    if (resting()) return;
    const all = candidates();
    if (all.length === 0 || random() > (TICK_MS / EVERY_MS) * all.length) return;
    const svg = all[Math.floor(random() * all.length)] as SVGSVGElement;
    let draw = draws.get(svg);
    if (!draw) {
      draw = { random: seeded(svg.dataset.bot ?? svg.dataset.color ?? ""), last: null };
      draws.set(svg, draw);
    }
    fidget(svg, nextFidget(draw));
  }, TICK_MS);
  return () => clearInterval(timer);
}
