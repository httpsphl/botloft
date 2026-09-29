import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { SidePanel } from "./SidePanel";

beforeEach(() => localStorage.clear());
afterEach(cleanup);

const panel = () => (
  <SidePanel label="Details" name="test" defaultWidth={320}>
    content
  </SidePanel>
);
const handle = () => screen.getByRole("separator", { name: "Resize panel" });
const width = () => Number(handle().getAttribute("aria-valuenow"));

describe("side panel", () => {
  test("arrow keys resize it, Shift steps further, and the width is kept", () => {
    const { unmount } = render(panel());
    expect(width()).toBe(320);
    fireEvent.keyDown(handle(), { key: "ArrowLeft" });
    expect(width()).toBe(344);
    fireEvent.keyDown(handle(), { key: "ArrowLeft", shiftKey: true });
    expect(width()).toBe(440);
    fireEvent.keyDown(handle(), { key: "ArrowRight" });
    expect(width()).toBe(416);
    expect(localStorage.getItem("botloft.panel.test")).toBe("416");

    unmount();
    render(panel());
    expect(width()).toBe(416);
  });

  test("it stays between its smallest and largest width, and double click resets it", () => {
    render(panel());
    for (let press = 0; press < 30; press += 1) {
      fireEvent.keyDown(handle(), { key: "ArrowRight", shiftKey: true });
    }
    expect(width()).toBe(256);
    for (let press = 0; press < 30; press += 1) {
      fireEvent.keyDown(handle(), { key: "ArrowLeft", shiftKey: true });
    }
    expect(width()).toBe(900);
    fireEvent.doubleClick(handle());
    expect(width()).toBe(320);
  });

  test("a saved width that makes no sense is ignored", () => {
    localStorage.setItem("botloft.panel.test", "12");
    render(panel());
    expect(width()).toBe(320);
  });
});
