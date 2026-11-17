// The working flame (spec 15.3): while its bot works, the mascot's flame
// burns on and on and throws embers. A CSS animation that runs forever
// makes the WebView draw every refresh of the screen, 180 a second on a
// fast monitor, so the flame runs none: one timer steps every working
// mascot together, FPS times a second, morphing the flame between the
// frames of flameFrames.ts and moving the embers up and away. Between two
// steps nothing changes and the WebView draws nothing. It stands still
// while the window is out of sight or has no focus, as the eyes do
// (eyeMoments.ts), and with less motion shows the flame as drawn. A mascot
// scrolled or folded out of view is not stepped at all. The same steps
// move the parts that go with the flame (ticks.ts).

import { type RefObject, useLayoutEffect } from "react";
import { lessMotion, resting } from "./eyeMoments";
import { WORKING_FRAMES } from "./flameFrames";
import { OUTLINE, SPARKS } from "./mascotArt";
import { settleParts, stepParts } from "./ticks";

/** Steps a second: smooth to the eye, a sixth of a fast screen's frames. */
export const FPS = 30;
/** One turn of the fire. */
const TURN_MS = 1500;
/** How often it looks again while it stands still. */
const REST_MS = 250;
/** Where an ember is brightest, as a share of its flight. */
const FLARE_AT = 0.18;

// Each frame as its numbers; the text between them is the same in all.
const NUMBER = /-?\d*\.?\d+/g;
const GLUE = OUTLINE.split(NUMBER);
const FRAMES = WORKING_FRAMES.map((frame) => (frame.match(NUMBER) ?? []).map(Number));

/** The flame at a point of its turn, from 0 to 1. */
export function flameAt(turn: number): string {
  const place = (((turn % 1) + 1) % 1) * FRAMES.length;
  const index = Math.floor(place);
  const from = FRAMES[index] ?? [];
  const to = FRAMES[(index + 1) % FRAMES.length] ?? [];
  const mix = place - index;
  let d = GLUE[0] ?? "";
  for (let i = 0; i < from.length; i++) {
    const a = from[i] ?? 0;
    const value = a + ((to[i] ?? a) - a) * mix;
    d += Math.round(value * 10) / 10 + (GLUE[i + 1] ?? "");
  }
  return d;
}

/** CSS `ease-out`, the curve from (0, 0) through (0.58, 1) to (1, 1). */
function easeOut(x: number): number {
  let low = 0;
  let high = 1;
  let s = x;
  for (let i = 0; i < 14; i++) {
    s = (low + high) / 2;
    const at = 3 * (1 - s) * s * s * 0.58 + s * s * s;
    if (at < x) low = s;
    else high = s;
  }
  return 3 * s * s - 2 * s * s * s;
}

const round = (value: number) => Math.round(value * 100) / 100;

/**
 * An ember at a point of its flight, from 0 to 1: it flares up, then
 * drifts off to one side as it fades.
 */
export function emberAt(
  flight: number,
  drift: number,
  turn: number,
): { opacity: number; transform: string } {
  const rising = flight < FLARE_AT;
  const mix = easeOut(rising ? flight / FLARE_AT : (flight - FLARE_AT) / (1 - FLARE_AT));
  const between = (from: number, to: number) => round(from + (to - from) * mix);
  const [x, y, scale, angle] = rising
    ? [between(0, drift * 0.15), between(0, -40), between(0.25, 1), between(0, turn * 0.3)]
    : [
        between(drift * 0.15, drift),
        between(-40, -200),
        between(1, 0.2),
        between(turn * 0.3, turn),
      ];
  return {
    opacity: rising ? between(0, 1) : between(1, 0),
    transform: `translate(${x}px, ${y}px) scale(${scale}) rotate(${angle}deg)`,
  };
}

interface Ember {
  path: SVGElement;
  delay: number;
  drift: number;
  turn: number;
}

interface Burning {
  svg: SVGSVGElement;
  /** On screen, as far as the browser said; unknown counts as on. */
  seen: boolean;
  fires: SVGPathElement[];
  embers: Ember[];
  /** Where in its turn this flame starts, so a crew's flames do not burn together. */
  offset: number;
}

const burning = new Map<SVGSVGElement, Burning>();
let timer: ReturnType<typeof setTimeout> | undefined;
let asDrawn = false;

/** Tells each flame whether it is on screen, where the browser can say. */
const watcher =
  typeof IntersectionObserver === "function"
    ? new IntersectionObserver((entries) => {
        for (const entry of entries) {
          const flame = burning.get(entry.target as SVGSVGElement);
          if (flame) flame.seen = entry.isIntersecting;
        }
      })
    : null;

function draw({ svg, seen, fires, embers, offset }: Burning, now: number): void {
  if (!seen) return;
  stepParts(svg, (now + offset) / 1000);
  const d = flameAt((now + offset) / TURN_MS);
  for (const fire of fires) fire.setAttribute("d", d);
  for (const { path, delay, drift, turn } of embers) {
    const flight = ((((now - delay) / TURN_MS) % 1) + 1) % 1;
    const ember = emberAt(flight, drift, turn);
    path.style.opacity = String(ember.opacity);
    path.style.transform = ember.transform;
  }
}

/** Back to the flame as drawn, with no embers. */
function settle({ svg, fires, embers }: Burning): void {
  settleParts(svg);
  for (const fire of fires) fire.setAttribute("d", OUTLINE);
  for (const { path } of embers) {
    path.style.removeProperty("opacity");
    path.style.removeProperty("transform");
  }
}

function step(): void {
  timer = undefined;
  if (burning.size === 0) return;
  if (lessMotion()) {
    if (!asDrawn) for (const flame of burning.values()) settle(flame);
    asDrawn = true;
  } else if (!resting()) {
    asDrawn = false;
    const now = performance.now();
    for (const flame of burning.values()) draw(flame, now);
  }
  timer = setTimeout(step, resting() ? REST_MS : 1000 / FPS);
}

/** Starts the flame of `svg` burning; returns what puts it out. */
export function burn(svg: SVGSVGElement, offset = 0): () => void {
  const flame: Burning = {
    svg,
    seen: true,
    fires: [...svg.querySelectorAll<SVGPathElement>(".avatar-fire")],
    embers: [...svg.querySelectorAll<SVGElement>(".avatar-spark")].map((path, index) => {
      const spark = SPARKS[index % SPARKS.length];
      return {
        path,
        delay: Number.parseFloat(spark?.delay ?? "0") * 1000,
        drift: Number.parseFloat(spark?.drift ?? "0"),
        turn: Number.parseFloat(spark?.turn ?? "0"),
      };
    }),
    offset,
  };
  burning.set(svg, flame);
  watcher?.observe(svg);
  if (!lessMotion()) draw(flame, performance.now());
  if (timer === undefined) timer = setTimeout(step, 1000 / FPS);
  return () => {
    burning.delete(svg);
    watcher?.unobserve(svg);
    settle(flame);
    if (burning.size === 0 && timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
  };
}

/** Keeps the flame of `svg` burning while `on`. */
export function useFlame(svg: RefObject<SVGSVGElement | null>, on: boolean, offset: number): void {
  useLayoutEffect(() => {
    if (!on || !svg.current) return;
    return burn(svg.current, offset);
  }, [svg, on, offset]);
}
