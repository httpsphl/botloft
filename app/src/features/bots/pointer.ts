// The mascot plays with the mouse (spec 15.3), with GSAP: with the cursor
// near, its open eyes follow it and spring back when it goes; under the
// cursor it squeezes, curious; clicked, it hops. Each tween goes on from
// where the last one left the part, so quick clicks never stall it. The
// body moves in `.avatar-poke`, inside the moments' `.avatar-spring`, and
// the eyes in `.avatar-gaze`, inside the CSS glance: the motions add up.
// One listener follows the cursor for every mascot, and only while it
// moves; with less motion or the window out of sight nothing reacts.

import { gsap } from "gsap";
import { type RefObject, useEffect } from "react";
import { windowHidden } from "../../shell/visibility";
import { lessMotion } from "./eyeMoments";
import { AT_FEET } from "./gestures";

/** How far the eyes move, in the artwork's units, as when glancing (useGlance.ts). */
const REACH = { x: 32, y: 22 };
/** How near the cursor must be, in avatar sizes from its middle, for the eyes to follow. */
const NEAR = 3;

const still = () => lessMotion() || windowHidden();

/** Its eyes are open: it is awake. */
const awake = (svg: SVGSVGElement) => !["idle", "sleeping"].includes(svg.dataset.mood ?? "");

const following = new Set<SVGSVGElement>();
const looking = new WeakSet<SVGSVGElement>();
let pending = false;
let cursor = { x: 0, y: 0 };

/** Turns the eyes of every mascot near the cursor toward it; the others look ahead again. */
function look(): void {
  pending = false;
  const off = still();
  for (const svg of following) {
    const gaze = svg.querySelector(".avatar-gaze");
    if (!gaze) continue;
    const box = svg.getBoundingClientRect();
    const size = Math.max(box.width, 1);
    const dx = cursor.x - (box.left + box.width / 2);
    const dy = cursor.y - (box.top + box.height / 2);
    if (!off && awake(svg) && Math.hypot(dx, dy) < size * NEAR) {
      const reach = (d: number) => Math.max(-1, Math.min(1, d / (size * 1.2)));
      looking.add(svg);
      gsap.to(gaze, {
        x: reach(dx) * REACH.x,
        y: reach(dy) * REACH.y,
        duration: 0.35,
        ease: "power2.out",
        overwrite: "auto",
      });
    } else if (looking.has(svg)) {
      lookAhead(svg);
    }
  }
}

function lookAhead(svg: SVGSVGElement): void {
  looking.delete(svg);
  const gaze = svg.querySelector(".avatar-gaze");
  if (gaze) gsap.to(gaze, { x: 0, y: 0, duration: 0.9, ease: "elastic.out(1, 0.45)" });
}

function moved(event: PointerEvent): void {
  cursor = { x: event.clientX, y: event.clientY };
  if (!pending) {
    pending = true;
    requestAnimationFrame(look);
  }
}

function left(): void {
  for (const svg of following) if (looking.has(svg)) lookAhead(svg);
}

/** Starts the eyes of `svg` following the cursor; returns what stops them. */
export function followCursor(svg: SVGSVGElement): () => void {
  if (following.size === 0) {
    document.addEventListener("pointermove", moved, { passive: true });
    document.documentElement.addEventListener("pointerleave", left);
  }
  following.add(svg);
  return () => {
    following.delete(svg);
    looking.delete(svg);
    const gaze = svg.querySelector(".avatar-gaze");
    if (gaze) gsap.killTweensOf(gaze);
    if (following.size === 0) {
      document.removeEventListener("pointermove", moved);
      document.documentElement.removeEventListener("pointerleave", left);
    }
  };
}

const poking = new WeakMap<Element, gsap.core.Timeline>();

/** The body of `svg` squeezes (`under` the cursor) or stands back up. */
export function squeeze(svg: SVGSVGElement, under: boolean): void {
  const poke = svg.querySelector(".avatar-poke");
  const hopping = poking.get(poke ?? svg);
  if (!poke || still() || (hopping && hopping.progress() < 1)) return;
  gsap.to(poke, {
    scaleX: under ? 1.08 : 1,
    scaleY: under ? 0.92 : 1,
    ...AT_FEET,
    duration: under ? 0.25 : 0.7,
    ease: under ? "back.out(3)" : "elastic.out(1, 0.4)",
    overwrite: "auto",
  });
}

/** `svg` hops, `height` times a full hop: a click, or an ember caught (toss.ts). */
export function hop(svg: SVGSVGElement, height = 1): void {
  const poke = svg.querySelector(".avatar-poke");
  if (!poke || still()) return;
  poking.get(poke)?.kill();
  const pose = (scaleX: number, scaleY: number, y = 0) => ({
    scaleX: 1 + (scaleX - 1) * height,
    scaleY: 1 + (scaleY - 1) * height,
    y: y * height,
    ...AT_FEET,
  });
  const tl = gsap
    .timeline()
    .to(poke, { ...pose(1.15, 0.85), duration: 0.08, ease: "power2.out" })
    .to(poke, { ...pose(0.9, 1.12, -70), duration: 0.18, ease: "power2.out" })
    .to(poke, { ...pose(1, 1, 0), duration: 0.16, ease: "power2.in" })
    .to(poke, { ...pose(1.12, 0.88), duration: 0.07, ease: "power1.out" })
    .to(poke, { ...pose(1, 1), duration: 0.6, ease: "elastic.out(1, 0.35)" });
  poking.set(poke, tl);
}

/** Lets the mascot drawn in `svg` play with the mouse while `on`. */
export function usePointerPlay(svg: RefObject<SVGSVGElement | null>, on: boolean): void {
  useEffect(() => {
    const drawn = svg.current;
    if (!on || !drawn) return;
    const stop = followCursor(drawn);
    const enter = () => squeeze(drawn, true);
    const leave = () => squeeze(drawn, false);
    const down = () => hop(drawn);
    drawn.addEventListener("pointerenter", enter);
    drawn.addEventListener("pointerleave", leave);
    drawn.addEventListener("pointerdown", down);
    return () => {
      stop();
      drawn.removeEventListener("pointerenter", enter);
      drawn.removeEventListener("pointerleave", leave);
      drawn.removeEventListener("pointerdown", down);
      const poke = drawn.querySelector(".avatar-poke");
      if (poke) gsap.killTweensOf(poke);
    };
  }, [svg, on]);
}
