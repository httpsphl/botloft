import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { burn, emberAt, FPS, flameAt } from "./flame";
import { WORKING_FRAMES } from "./flameFrames";
import { OUTLINE } from "./mascotArt";

const SVG = "http://www.w3.org/2000/svg";
let stop: () => void = () => {};

/** A working mascot: its flame (clip and edge) and its embers. */
function mascot(): SVGSVGElement {
  const svg = document.createElementNS(SVG, "svg");
  for (const className of ["avatar-fire", "avatar-fire", "avatar-spark", "avatar-spark"]) {
    const path = document.createElementNS(SVG, "path");
    path.setAttribute("class", className);
    path.setAttribute("d", OUTLINE);
    svg.append(path);
  }
  document.body.append(svg);
  return svg;
}

const fires = (svg: SVGSVGElement) =>
  [...svg.querySelectorAll(".avatar-fire")].map((fire) => fire.getAttribute("d"));

const numbers = (d: string) => (d.match(/-?\d*\.?\d+/g) ?? []).map(Number);

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "performance"] });
});

afterEach(() => {
  stop();
  document.body.replaceChildren();
  document.documentElement.removeAttribute("data-blurred");
  document.documentElement.removeAttribute("data-motion");
  vi.useRealTimers();
});

describe("the working flame", () => {
  test("passes through every frame as drawn, morphing between them", () => {
    const steps = WORKING_FRAMES.length;
    WORKING_FRAMES.forEach((frame, index) => {
      expect(flameAt(index / steps)).toBe(frame);
    });
    expect(flameAt(1)).toBe(WORKING_FRAMES[0]);

    const from = numbers(WORKING_FRAMES[0]);
    const to = numbers(WORKING_FRAMES[1]);
    const between = numbers(flameAt(0.5 / steps));
    between.forEach((value, i) => {
      const [low, high] = [from[i] ?? 0, to[i] ?? 0].sort((a, b) => a - b);
      expect(value).toBeGreaterThanOrEqual((low ?? 0) - 0.05);
      expect(value).toBeLessThanOrEqual((high ?? 0) + 0.05);
    });
  });

  test("an ember flares up, then drifts off and fades", () => {
    expect(emberAt(0, 20, 10).opacity).toBe(0);
    expect(emberAt(0.18, 20, 10)).toEqual({
      opacity: 1,
      transform: "translate(3px, -40px) scale(1) rotate(3deg)",
    });
    expect(emberAt(0.999, 20, 10).opacity).toBeLessThan(0.01);
  });

  test("steps a few times a second while it burns, and goes out as drawn", () => {
    const svg = mascot();
    stop = burn(svg);
    const first = fires(svg);
    expect(first[0]).not.toBe(OUTLINE);
    expect(first[1]).toBe(first[0]);

    vi.advanceTimersByTime(1000 / FPS);
    expect(fires(svg)[0]).not.toBe(first[0]);
    expect(svg.querySelector<SVGElement>(".avatar-spark")?.style.transform).toContain("translate");

    stop();
    expect(fires(svg)).toEqual([OUTLINE, OUTLINE]);
    expect(svg.querySelector<SVGElement>(".avatar-spark")?.style.transform).toBe("");
    expect(vi.getTimerCount()).toBe(0);
  });

  test("stands still while the window has no focus", () => {
    const svg = mascot();
    stop = burn(svg);
    document.documentElement.setAttribute("data-blurred", "");
    vi.advanceTimersByTime(1000 / FPS);
    const held = fires(svg)[0];
    vi.advanceTimersByTime(2000);
    expect(fires(svg)[0]).toBe(held);

    document.documentElement.removeAttribute("data-blurred");
    vi.advanceTimersByTime(1000);
    expect(fires(svg)[0]).not.toBe(held);
  });

  test("shows the flame as drawn with less motion", () => {
    document.documentElement.setAttribute("data-motion", "less");
    const svg = mascot();
    stop = burn(svg);
    vi.advanceTimersByTime(1000);
    expect(fires(svg)).toEqual([OUTLINE, OUTLINE]);
  });
});
