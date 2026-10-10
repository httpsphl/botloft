import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { Reel } from "./Reel";

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

test("the agent in the middle lands, works, cheers, then the row moves on", () => {
  const { container } = render(<Reel />);
  const middle = () => container.querySelector("[data-center] svg") as SVGElement;
  const color = () => middle().querySelector("rect")?.getAttribute("fill");
  const first = color();
  expect(container.querySelectorAll(".reel-place")).toHaveLength(9);
  expect(middle().dataset.mood).toBe("idle");

  act(() => vi.advanceTimersByTime(500));
  expect(middle().dataset.mood).toBe("working");

  act(() => vi.advanceTimersByTime(1100));
  expect(middle().dataset.mood).toBe("idle");
  expect(middle().hasAttribute("data-cheer")).toBe(true);

  act(() => vi.advanceTimersByTime(1400));
  expect(color()).not.toBe(first);
  expect(middle().dataset.mood).toBe("idle");
});
