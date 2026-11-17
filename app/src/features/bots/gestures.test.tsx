import { cleanup, render } from "@testing-library/react";
import { gsap } from "gsap";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { BotId } from "../../lib/protocol.gen";
import { APP_OPENED } from "../../ui/motion";
import { BotAvatar } from "./BotAvatar";
import { gestureOf } from "./gestures";
import { ListAvatar } from "./ListAvatar";

afterEach(() => {
  cleanup();
  document.documentElement.removeAttribute("data-motion");
  vi.useRealTimers();
});

const svgOf = (container: HTMLElement) => container.querySelector("svg") as SVGSVGElement;
const spring = (svg: SVGSVGElement) => svg.querySelector(".avatar-spring") as SVGGElement;
/** The body's transform: an attribute in a browser, a style where the DOM knows no SVG geometry. */
const pose = (svg: SVGSVGElement) =>
  spring(svg).getAttribute("transform") || spring(svg).style.transform || null;
const eyes = (svg: SVGSVGElement) => [...svg.querySelectorAll<SVGElement>(".avatar-eye")];

describe("gestures", () => {
  test("a cheer plays, then gives the mascot back to its mood as it was", () => {
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="working" />);
    const svg = svgOf(container);
    const smile = svg.querySelector(".avatar-happy") as SVGPathElement;
    const drawn = smile.getAttribute("d");
    rerender(<BotAvatar color="#5ec8ff" mood="idle" />);

    const cheer = gestureOf(svg);
    expect(cheer).toBeDefined();
    expect(svg.hasAttribute("data-gesture")).toBe(true);
    cheer?.progress(0.5);
    expect(pose(svg)).not.toBeNull();
    expect(smile.style.opacity).toBe("1");

    cheer?.progress(1);
    expect(svg.hasAttribute("data-gesture")).toBe(false);
    expect(pose(svg)).toBeNull();
    for (const eye of eyes(svg)) {
      expect(eye.style.opacity).toBe("");
      expect(eye.style.clipPath).toBe("");
    }
    expect(smile.getAttribute("d")).toBe(drawn);
  });

  test("a gesture cut short hands its pose to the next instead of jumping", () => {
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="sleeping" />);
    const svg = svgOf(container);
    rerender(<BotAvatar color="#5ec8ff" mood="working" />);
    const wake = gestureOf(svg);
    wake?.progress(0.3);
    const halfway = pose(svg);
    expect(halfway).not.toBeNull();

    rerender(<BotAvatar color="#5ec8ff" mood="idle" />);
    expect(gestureOf(svg)).not.toBe(wake);
    expect(gsap.globalTimeline.getChildren(false).includes(wake as gsap.core.Timeline)).toBe(false);
    expect(pose(svg)).toBe(halfway);
  });

  test("with less motion no gesture plays", () => {
    document.documentElement.setAttribute("data-motion", "less");
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="working" />);
    rerender(<BotAvatar color="#5ec8ff" mood="idle" />);
    expect(gestureOf(svgOf(container))).toBeUndefined();
    expect(svgOf(container).hasAttribute("data-gesture")).toBe(false);
  });

  test("a held-still or moodless mascot makes no gesture", () => {
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" mood="working" still />);
    rerender(<BotAvatar color="#5ec8ff" mood="idle" still />);
    expect(gestureOf(svgOf(container))).toBeUndefined();
  });

  test("a new agent pops in, and starting up meanwhile does not wake it on top", () => {
    vi.useFakeTimers();
    vi.setSystemTime(APP_OPENED + 60_000);
    const created = { id: "bot_new" as BotId, color: "#9be564", paused: false };
    const createdAt = APP_OPENED + 59_000;
    const { container, rerender } = render(
      <ListAvatar bot={{ ...created, state: "offline", createdAt }} crewPaused={false} size={32} />,
    );
    const svg = svgOf(container);
    const pop = gestureOf(svg);
    expect(pop).toBeDefined();
    pop?.progress(0.1);
    expect(spring(svg).style.opacity).not.toBe("1");

    rerender(
      <ListAvatar
        bot={{ ...created, state: "launching", createdAt }}
        crewPaused={false}
        size={32}
      />,
    );
    expect(gestureOf(svg)).toBe(pop);
  });
});
