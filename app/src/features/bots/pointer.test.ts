import { gsap } from "gsap";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { followCursor, hop, squeeze } from "./pointer";

const SVG = "http://www.w3.org/2000/svg";
let stops: (() => void)[] = [];

/** A 32 px mascot at (100, 100), with its eyes and body groups. */
function mascot(mood = "awake", left = 100): SVGSVGElement {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("data-mood", mood);
  for (const className of ["avatar-poke", "avatar-gaze"]) {
    const g = document.createElementNS(SVG, "g");
    g.setAttribute("class", className);
    svg.append(g);
  }
  svg.getBoundingClientRect = () => new DOMRect(left, 100, 32, 32);
  document.body.append(svg);
  stops.push(followCursor(svg));
  return svg;
}

const gaze = (svg: SVGSVGElement) => svg.querySelector(".avatar-gaze") as Element;
const poke = (svg: SVGSVGElement) => svg.querySelector(".avatar-poke") as Element;
/** Where the last tween started on `target` is heading. */
const heading = (target: Element) => gsap.getTweensOf(target).at(-1)?.vars;

function moveTo(x: number, y: number): void {
  document.dispatchEvent(new MouseEvent("pointermove", { clientX: x, clientY: y }));
}

beforeEach(() => {
  vi.stubGlobal("requestAnimationFrame", (run: FrameRequestCallback) => {
    run(0);
    return 0;
  });
});

afterEach(() => {
  for (const stop of stops) stop();
  stops = [];
  document.body.replaceChildren();
  document.documentElement.removeAttribute("data-motion");
  vi.unstubAllGlobals();
});

describe("playing with the mouse", () => {
  test("the eyes of a mascot near the cursor turn toward it, and spring back when it goes", () => {
    const svg = mascot();
    moveTo(160, 116);
    expect(heading(gaze(svg))).toMatchObject({ x: 32, y: 0 });

    moveTo(116, 60);
    expect(heading(gaze(svg))?.y).toBeLessThan(0);

    moveTo(900, 900);
    expect(heading(gaze(svg))).toMatchObject({ x: 0, y: 0 });
  });

  test("a far, a sleeping or a held mascot keeps its eyes ahead", () => {
    const far = mascot("awake", 600);
    const sleeping = mascot("sleeping", 140);
    moveTo(150, 116);
    expect(gsap.getTweensOf(gaze(far))).toHaveLength(0);
    expect(gsap.getTweensOf(gaze(sleeping))).toHaveLength(0);

    document.documentElement.setAttribute("data-motion", "less");
    const near = mascot("awake", 100);
    moveTo(120, 116);
    expect(gsap.getTweensOf(gaze(near))).toHaveLength(0);
  });

  test("it squeezes under the cursor, and a click makes it hop from where it is", () => {
    const svg = mascot();
    squeeze(svg, true);
    expect(heading(poke(svg))).toMatchObject({ scaleX: 1.08, scaleY: 0.92 });

    hop(svg);
    const first = gsap.getTweensOf(poke(svg)).length;
    hop(svg);
    // The second hop replaces the first instead of queueing after it.
    expect(gsap.getTweensOf(poke(svg)).length).toBe(first);

    // While it hops, the cursor does not squeeze it.
    squeeze(svg, false);
    expect(gsap.getTweensOf(poke(svg)).length).toBe(first);
  });

  test("with less motion it does not react", () => {
    document.documentElement.setAttribute("data-motion", "less");
    const svg = mascot();
    squeeze(svg, true);
    hop(svg);
    expect(gsap.getTweensOf(poke(svg))).toHaveLength(0);
  });
});
