// The player of the site's scenes (spec 39.4): parts show on their steps,
// mascots change mood, text writes itself out, scenes loop and hold, and a
// visitor who asks for less motion sees each at its end.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { moodAt, playScenes, shownAt, showStep, timingOf } from "./player";

function scene(steps = 3): HTMLElement {
  const root = document.createElement("div");
  root.innerHTML = `
    <div data-scene data-steps="${steps}" data-beat="1000" data-hold="2000">
      <p class="first" data-at="1">asked</p>
      <p class="card" data-at="1" data-until="2">card</p>
      <p class="typed" data-at="2" data-type>Found three sources</p>
      <span class="mascot" data-mood="awake" data-moods="0:awake,2:working"></span>
    </div>`;
  return root;
}

const shown = (root: HTMLElement, selector: string) =>
  root.querySelector(selector)?.classList.contains("is-on");

describe("a scene's steps", () => {
  it("show a part from its step and until its end", () => {
    expect(shownAt(0, 1, null)).toBe(false);
    expect(shownAt(1, 1, null)).toBe(true);
    expect(shownAt(2, 1, 2)).toBe(false);
    expect(shownAt(0, null, 2)).toBe(true);
  });

  it("give each mascot the mood due on that step", () => {
    expect(moodAt("0:awake,2:working,5:awake", 0)).toBe("awake");
    expect(moodAt("0:awake,2:working,5:awake", 3)).toBe("working");
    expect(moodAt("0:awake,2:working,5:awake", 9)).toBe("awake");
    expect(moodAt("2:working", 1)).toBeNull();
    expect(moodAt("", 1)).toBeNull();
  });

  it("read their timing, with defaults for what is missing", () => {
    const root = scene(5);
    const element = root.querySelector<HTMLElement>("[data-scene]");
    expect(element && timingOf(element)).toEqual({ steps: 5, beat: 1000, hold: 2000 });
    expect(timingOf(document.createElement("div"))).toEqual({ steps: 1, beat: 1400, hold: 3200 });
  });

  it("mark the parts on, write text out and tell a mascot its new mood", () => {
    const root = scene();
    const element = root.querySelector<HTMLElement>("[data-scene]") as HTMLElement;
    const moods: string[] = [];
    root.querySelector(".mascot")?.addEventListener("site:mood", (event) => {
      moods.push((event as CustomEvent<string>).detail);
    });
    const typing = vi.fn();
    showStep(element, 1, typing);
    expect(element.dataset.step).toBe("1");
    expect(shown(root, ".first")).toBe(true);
    expect(shown(root, ".card")).toBe(true);
    expect(shown(root, ".typed")).toBe(false);
    showStep(element, 2, typing);
    expect(shown(root, ".card")).toBe(false);
    expect(typing).toHaveBeenCalledTimes(1);
    expect(moods).toEqual(["working"]);
    // A part already shown is not written out again.
    showStep(element, 2, typing);
    expect(typing).toHaveBeenCalledTimes(1);
  });
});

describe("playing the scenes", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("moves one step per beat, holds at the end and starts again", () => {
    const root = scene();
    const element = root.querySelector<HTMLElement>("[data-scene]") as HTMLElement;
    const stop = playScenes(root, false);
    expect(element.dataset.step).toBe("0");
    vi.advanceTimersByTime(1000);
    expect(element.dataset.step).toBe("1");
    vi.advanceTimersByTime(1000);
    expect(element.dataset.step).toBe("2");
    vi.advanceTimersByTime(1000);
    expect(element.dataset.step).toBe("2");
    vi.advanceTimersByTime(1000);
    expect(element.dataset.step).toBe("0");
    // The text written out is whole again for the next turn.
    expect(root.querySelector(".typed")?.textContent).toBe("Found three sources");
    stop();
    vi.advanceTimersByTime(5000);
    expect(element.dataset.step).toBe("0");
  });

  it("shows each scene at its end, still, for less motion", () => {
    const root = scene();
    const element = root.querySelector<HTMLElement>("[data-scene]") as HTMLElement;
    playScenes(root, true);
    expect(element.dataset.step).toBe("2");
    expect(shown(root, ".typed")).toBe(true);
    expect(shown(root, ".card")).toBe(false);
    vi.advanceTimersByTime(10_000);
    expect(element.dataset.step).toBe("2");
  });
});
