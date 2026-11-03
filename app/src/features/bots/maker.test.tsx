import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { BotAvatar, MAKER_COLOR } from "./BotAvatar";
import { gestureOf } from "./gestures";
import { settleParts, stepParts } from "./ticks";

afterEach(cleanup);

const svgOf = (container: HTMLElement) => container.querySelector("svg") as SVGSVGElement;

describe("the Maker's mascot", () => {
  test("is the flame in white, with its wand from 72 px and the flow only with room", () => {
    const { container, rerender } = render(<BotAvatar color="#5ec8ff" size={28} maker />);
    expect(container.querySelector(".maker-wand")).toBeNull();
    expect(svgOf(container).dataset.color).toBe(MAKER_COLOR);
    // Every color of the flame is a gray near white; the eyes keep their glow.
    const body = container.querySelector("[clip-path] rect")?.getAttribute("fill") ?? "";
    const [r, g, b] = [1, 3, 5].map((at) => Number.parseInt(body.slice(at, at + 2), 16));
    expect(r).toBe(g);
    expect(g).toBe(b);
    expect(r).toBeGreaterThan(230);

    rerender(<BotAvatar color="#5ec8ff" size={96} maker />);
    expect(container.querySelector(".maker-wand")).not.toBeNull();
    expect(container.querySelector(".maker-flow")).toBeNull();
    // The wand widens the box, never the mascot.
    expect(Number(svgOf(container).getAttribute("height"))).toBe(96);
    expect(Number(svgOf(container).getAttribute("width"))).toBeGreaterThan(96);

    rerender(<BotAvatar color="#5ec8ff" size={120} maker scene />);
    expect(container.querySelectorAll(".maker-box")).toHaveLength(3);
  });

  test("an agent never wears the Maker's gear", () => {
    const { container } = render(<BotAvatar color="#5ec8ff" size={96} mood="working" />);
    expect(container.querySelector(".maker-wand, .maker-flow")).toBeNull();
  });

  test("working, its flow lights one box at a time; settled, it is as drawn", () => {
    const { container } = render(<BotAvatar color="#ff7a59" size={120} maker scene />);
    const svg = svgOf(container);
    const lit = () =>
      [...svg.querySelectorAll(".maker-box")].map((box) => box.getAttribute("data-lit"));
    stepParts(svg, 0.1);
    expect(lit()).toEqual(["", "off", "off"]);
    stepParts(svg, 1.0);
    expect(lit()).toEqual(["off", "", "off"]);
    settleParts(svg);
    expect(lit()).toEqual([null, null, null]);
    // Each box keeps its place in the flow.
    const places = [...svg.querySelectorAll(".maker-box")].map((box) =>
      box.parentElement?.getAttribute("transform"),
    );
    expect(new Set(places).size).toBe(3);
  });

  test("it cheers with a twirl and its whole flow lit, then settles", () => {
    vi.useFakeTimers();
    const { container, rerender } = render(
      <BotAvatar color="#ff7a59" size={120} mood="working" maker scene />,
    );
    rerender(<BotAvatar color="#ff7a59" size={120} mood="idle" maker scene />);
    const svg = svgOf(container);
    const cheer = gestureOf(svg);
    cheer?.progress(0.5);
    const boxes = [...svg.querySelectorAll(".maker-box")];
    expect(boxes.every((box) => box.getAttribute("data-lit") === "")).toBe(true);
    expect(svg.querySelector('.maker-wand [data-tick="wave"]')?.getAttribute("transform")).toMatch(
      /rotate/,
    );
    cheer?.progress(1);
    expect(boxes.every((box) => !box.hasAttribute("data-lit"))).toBe(true);
    vi.useRealTimers();
  });
});
