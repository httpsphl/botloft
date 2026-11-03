// The mascot's moments as gestures (spec 15.3), played with GSAP: it
// cheers when its bot finishes, wakes when it has something to do, falls
// asleep when it has nothing left, and pops in when its bot was just
// created (mascotMoments.ts says when). Each gesture is a timeline on the
// body (`.avatar-spring`, squashed and stretched from the bottom of the
// flame) and on the eyes. Every step tweens from where the parts are now,
// so a gesture cut short by the next one carries on from there instead of
// jumping. Done, it clears what it set and the mood's CSS takes over,
// which shows the same: every gesture ends at rest. While the mascots
// stand still (eyeMoments.ts) none plays. The flame's shape moves with the
// `d` transition in mascot.css.

import { gsap } from "gsap";
import { type RefObject, useEffect, useLayoutEffect, useRef } from "react";
import { resting } from "./eyeMoments";
import { ARRIVE_MS, MOMENT_MS, type Moment } from "./mascotMoments";

/**
 * The bottom middle of the body, in the artwork's units: the body squashes
 * onto it. It never moves, so GSAP need not shift the body to make up for
 * a new origin (`smoothOrigin`), which left it off by a few units.
 */
export const AT_FEET = { svgOrigin: "306 740", smoothOrigin: false };

/** How the eyes look, as `clip-path`: open, or shut down to a line in the middle. */
const OPEN = "inset(0% 0% 0% 0%)";
const shut = (share: number) => `inset(${share}% 0% ${share}% 0%)`;

interface Parts {
  spring: Element | null;
  eyes: Element[];
  closed: Element[];
  smiles: SVGPathElement[];
}

function partsOf(svg: SVGSVGElement): Parts {
  return {
    spring: svg.querySelector(".avatar-spring"),
    eyes: [...svg.querySelectorAll(".avatar-eye")],
    closed: [...svg.querySelectorAll(".avatar-closed")],
    smiles: [...svg.querySelectorAll<SVGPathElement>(".avatar-happy")],
  };
}

/** A smile pressed flat by `flat` (0 to 1) toward the line between its ends. */
function pressed(d: string, flat: number): string {
  const [x0, y0, cx, cy, x1, y1] = (d.match(/-?\d*\.?\d+/g) ?? []).map(Number);
  const y = (y0 ?? 0) + ((cy ?? 0) - (y0 ?? 0)) * (1 - flat);
  return `M${x0} ${y0}Q${cx} ${y} ${x1} ${y1}`;
}

const playing = new WeakMap<SVGSVGElement, gsap.core.Timeline>();
const smileOf = new WeakMap<SVGPathElement, string>();

/** The gesture `svg` is in, if any. */
export function gestureOf(svg: SVGSVGElement): gsap.core.Timeline | undefined {
  return playing.get(svg);
}

/** Ends any gesture of `svg` and gives it back to its mood. */
export function settle(svg: SVGSVGElement): void {
  playing.get(svg)?.kill();
  playing.delete(svg);
  const { spring, eyes, closed, smiles } = partsOf(svg);
  if (spring) gsap.set(spring, { clearProps: "all" });
  gsap.set([...eyes, ...closed, ...smiles], { clearProps: "opacity,clipPath" });
  for (const smile of smiles) {
    const d = smileOf.get(smile);
    if (d) smile.setAttribute("d", d);
  }
  svg.removeAttribute("data-gesture");
}

/** Plays a gesture on `svg`, on from where the one before left it. */
function play(svg: SVGSVGElement, build: (tl: gsap.core.Timeline, parts: Parts) => void): void {
  playing.get(svg)?.kill();
  if (resting()) {
    settle(svg);
    return;
  }
  const parts = partsOf(svg);
  for (const smile of parts.smiles) {
    if (!smileOf.has(smile)) smileOf.set(smile, smile.getAttribute("d") ?? "");
  }
  svg.setAttribute("data-gesture", "");
  const tl = gsap.timeline({ onComplete: () => settle(svg) });
  build(tl, parts);
  playing.set(svg, tl);
}

type Pose = [scaleX: number, scaleY: number, y?: number];

/** The body through `poses`, each reached at its time (seconds) with its ease. */
function body(tl: gsap.core.Timeline, spring: Element | null, poses: [number, Pose, string][]) {
  if (!spring) return;
  let at = 0;
  for (const [until, [scaleX, scaleY, y = 0], ease] of poses) {
    tl.to(spring, { scaleX, scaleY, y, ...AT_FEET, duration: until - at, ease }, at);
    at = until;
  }
}

/** Crouches, jumps up stretched, lands squashed and settles; the eyes smile meanwhile. */
function cheer(tl: gsap.core.Timeline, { spring, eyes, smiles }: Parts): void {
  const s = MOMENT_MS.cheer / 1000;
  body(tl, spring, [
    [0.12 * s, [1.1, 0.9], "power1.inOut"],
    [0.3 * s, [0.93, 1.08, -98], "power2.out"],
    [0.46 * s, [1.1, 0.9], "power2.in"],
    [0.58 * s, [0.97, 1.03, -18], "power1.out"],
    [0.7 * s, [1, 1], "power1.inOut"],
  ]);
  tl.to(eyes, { opacity: 0, clipPath: shut(50), duration: 0.04 * s }, 0.2 * s);
  for (const smile of smiles) {
    const d = smileOf.get(smile) ?? "";
    tl.set(smile, { attr: { d: pressed(d, 0.6) } }, 0)
      .to(smile, { opacity: 1, duration: 0.04 * s }, 0.2 * s)
      .to(smile, { attr: { d }, duration: 0.08 * s, ease: "back.out(2)" }, 0.24 * s)
      .to(smile, { opacity: 0, duration: 0.04 * s }, 0.82 * s);
  }
  tl.to(eyes, { opacity: 1, clipPath: OPEN, duration: 0.08 * s, ease: "power2.out" }, 0.84 * s);
}

/** The ember grows past the flame and settles; the eyes open halfway, blink heavily, open. */
function wake(tl: gsap.core.Timeline, { spring, eyes, closed }: Parts): void {
  const s = MOMENT_MS.wake / 1000;
  body(tl, spring, [
    [0.06 * s, [0.8, 0.62], "power1.out"],
    [0.4 * s, [0.95, 1.1], "power2.out"],
    [0.62 * s, [1.04, 0.95], "sine.inOut"],
    [0.8 * s, [0.99, 1.01], "sine.inOut"],
    [s, [1, 1], "sine.inOut"],
  ]);
  tl.to(eyes, { opacity: 0, clipPath: shut(50), duration: 0.06 * s }, 0)
    .to(eyes, { opacity: 1, clipPath: shut(28), duration: 0.18 * s }, 0.3 * s)
    .to(eyes, { clipPath: shut(46), duration: 0.12 * s }, 0.48 * s)
    .to(eyes, { clipPath: OPEN, duration: 0.18 * s, ease: "power2.out" }, 0.6 * s);
  tl.to(closed, { opacity: 1, duration: 0.06 * s }, 0).to(
    closed,
    { opacity: 0, duration: 0.06 * s },
    0.28 * s,
  );
}

/** Yawns up, the lids grow heavy, open a little once more and shut into the arcs. */
function sleep(tl: gsap.core.Timeline, { spring, eyes, closed }: Parts): void {
  const s = MOMENT_MS.sleep / 1000;
  body(tl, spring, [
    [0.25 * s, [0.97, 1.05], "power1.out"],
    [0.6 * s, [1.05, 0.94], "sine.inOut"],
    [0.8 * s, [0.99, 1.01], "sine.inOut"],
    [s, [1, 1], "sine.inOut"],
  ]);
  tl.set(eyes, { opacity: 1 }, 0)
    .to(eyes, { clipPath: shut(32), duration: 0.3 * s, ease: "sine.inOut" }, 0)
    .to(eyes, { clipPath: shut(20), duration: 0.15 * s }, 0.3 * s)
    .to(eyes, { clipPath: shut(50), duration: 0.27 * s, ease: "power2.in" }, 0.45 * s)
    .to(eyes, { opacity: 0, duration: 0.06 * s }, 0.72 * s);
  tl.set(closed, { opacity: 0 }, 0).to(closed, { opacity: 1, duration: 0.08 * s }, 0.72 * s);
}

/** Grows from a dot, lands squashed, springs up and settles; the eyes open unless asleep. */
function arrive(eyesOpen: boolean) {
  return (tl: gsap.core.Timeline, { spring, eyes }: Parts) => {
    const s = ARRIVE_MS / 1000;
    if (spring) tl.set(spring, { opacity: 0, scaleX: 0.2, scaleY: 0.2, ...AT_FEET }, 0);
    if (spring) tl.to(spring, { opacity: 1, duration: 0.2 * s }, 0);
    body(tl, spring, [
      [0.35 * s, [1.12, 0.88], "power3.out"],
      [0.58 * s, [0.95, 1.06], "sine.inOut"],
      [0.78 * s, [1.02, 0.98], "sine.inOut"],
      [s, [1, 1], "sine.inOut"],
    ]);
    if (eyesOpen) {
      tl.set(eyes, { clipPath: shut(50) }, 0).to(
        eyes,
        { clipPath: OPEN, duration: 0.2 * s, ease: "power2.out" },
        0.4 * s,
      );
    }
  };
}

const MOMENTS: Record<Moment, (tl: gsap.core.Timeline, parts: Parts) => void> = {
  cheer,
  wake,
  sleep,
};

/**
 * Plays the gesture of each moment of the mascot drawn in `svg`: the pop
 * of a bot just created, then the moment its moods call for. A bot that
 * starts up while it pops in does not wake on top of it.
 */
export function useGestures(
  svg: RefObject<SVGSVGElement | null>,
  moment: Moment | null,
  arriving: boolean,
  asleep: boolean,
): void {
  const popping = useRef(false);
  // Whether it sleeps only matters as the pop starts.
  const sleepy = useRef(asleep);
  sleepy.current = asleep;
  useLayoutEffect(() => {
    popping.current = arriving;
    if (arriving && svg.current) play(svg.current, arrive(!sleepy.current));
  }, [svg, arriving]);
  useLayoutEffect(() => {
    if (!moment || !svg.current) return;
    if (moment === "wake" && popping.current) return;
    play(svg.current, MOMENTS[moment]);
  }, [svg, moment]);
  useEffect(() => {
    const drawn = svg.current;
    return () => {
      if (drawn) playing.get(drawn)?.kill();
    };
  }, [svg]);
}
