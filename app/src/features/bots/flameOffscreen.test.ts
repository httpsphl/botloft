import { afterEach, describe, expect, test, vi } from "vitest";
import { OUTLINE } from "./mascotArt";

const SVG = "http://www.w3.org/2000/svg";

/** An observer that reports what the test says, to the last one made. */
let report: (target: Element, isIntersecting: boolean) => void = () => {};
class FakeObserver {
  constructor(callback: (entries: { target: Element; isIntersecting: boolean }[]) => void) {
    report = (target, isIntersecting) => callback([{ target, isIntersecting }]);
  }
  observe() {}
  unobserve() {}
  disconnect() {}
}

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.resetModules();
  document.body.replaceChildren();
});

describe("a flame off screen", () => {
  test("is not stepped until it shows again", async () => {
    vi.stubGlobal("IntersectionObserver", FakeObserver);
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "performance"] });
    const { burn, FPS } = await import("./flame");
    const svg = document.createElementNS(SVG, "svg");
    const fire = document.createElementNS(SVG, "path");
    fire.setAttribute("class", "avatar-fire");
    fire.setAttribute("d", OUTLINE);
    svg.append(fire);
    document.body.append(svg);
    const stop = burn(svg);

    report(svg, false);
    const held = fire.getAttribute("d");
    vi.advanceTimersByTime(1000);
    expect(fire.getAttribute("d")).toBe(held);

    report(svg, true);
    vi.advanceTimersByTime(1000 / FPS);
    expect(fire.getAttribute("d")).not.toBe(held);
    stop();
  });
});
