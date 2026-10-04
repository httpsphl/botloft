import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { startEyeMoments } from "./eyeMoments";

let stop: () => void = () => {};

function mascot(mood: string, still = false): SVGElement {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("data-mood", mood);
  if (still) svg.setAttribute("data-still", "");
  svg.getBoundingClientRect = () =>
    ({ width: 28, height: 28, top: 10, bottom: 38, left: 0, right: 28 }) as DOMRect;
  document.body.append(svg);
  return svg;
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  stop();
  document.body.replaceChildren();
  document.documentElement.removeAttribute("data-blurred");
  vi.useRealTimers();
});

describe("the eyes' moments", () => {
  test("awake mascots blink for a moment, then stand still", () => {
    const idle = mascot("idle");
    const sleeping = mascot("sleeping");
    const still = mascot("idle", true);
    // Every draw says yes, with no spread.
    stop = startEyeMoments(() => 0);
    // A moment starts on its own timer, right after the tick.
    vi.advanceTimersByTime(2001);
    expect(idle.hasAttribute("data-blink")).toBe(true);
    expect(sleeping.hasAttribute("data-blink")).toBe(false);
    expect(still.hasAttribute("data-blink")).toBe(false);
    vi.advanceTimersByTime(300);
    expect(idle.hasAttribute("data-blink")).toBe(false);
  });

  test("one idle mascot looks around now and then, not on every tick", () => {
    const idle = mascot("idle");
    const working = mascot("working");
    stop = startEyeMoments(() => 0.99);
    vi.advanceTimersByTime(8000);
    expect(idle.hasAttribute("data-look")).toBe(false);
    vi.advanceTimersByTime(2001);
    expect(idle.hasAttribute("data-look")).toBe(true);
    expect(working.hasAttribute("data-look")).toBe(false);
    vi.advanceTimersByTime(4500);
    expect(idle.hasAttribute("data-look")).toBe(false);
  });

  test("nothing moves while the window has no focus", () => {
    const idle = mascot("idle");
    document.documentElement.setAttribute("data-blurred", "");
    stop = startEyeMoments(() => 0);
    vi.advanceTimersByTime(20_000);
    expect(idle.hasAttribute("data-blink")).toBe(false);
    expect(idle.hasAttribute("data-look")).toBe(false);
  });
});
