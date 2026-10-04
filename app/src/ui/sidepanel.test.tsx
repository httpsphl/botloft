import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { PanelClosing, PanelRestored } from "./panelMotion";
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

/**
 * The animation named `name` ends on the panel itself. jsdom has no
 * AnimationEvent, so React listens for the prefixed name there.
 */
function ends(element: HTMLElement, name: string) {
  const event = new Event("webkitAnimationEnd", { bubbles: true });
  Object.defineProperty(event, "animationName", { value: name });
  act(() => {
    fireEvent(element, event);
  });
}

describe("side panel motion", () => {
  test("slides open, then stays open", () => {
    const { container } = render(panel());
    const aside = container.querySelector("aside") as HTMLElement;
    expect(aside.className).toContain("panel-opening");
    ends(aside, "panel-open");
    expect(aside.className).not.toContain("panel-opening");
  });

  test("one that comes back with its view is open at once, and still slides closed", () => {
    const closed = vi.fn();
    const back = (closing: { closed(): void } | null) => (
      <PanelRestored.Provider value>
        <PanelClosing.Provider value={closing}>{panel()}</PanelClosing.Provider>
      </PanelRestored.Provider>
    );
    const view = render(back(null));
    const aside = view.container.querySelector("aside") as HTMLElement;
    expect(aside.className).not.toContain("panel-opening");
    expect(screen.getByRole("complementary", { name: "Details" })).toBeDefined();
    view.rerender(back({ closed }));
    expect(aside.className).toContain("panel-closing");
    ends(aside, "panel-close");
    expect(closed).toHaveBeenCalled();
  });

  test("while closing it leaves the page to the rest, and goes when the slide ends", () => {
    const closed = vi.fn();
    const { container } = render(
      <PanelClosing.Provider value={{ closed }}>{panel()}</PanelClosing.Provider>,
    );
    const aside = container.querySelector("aside") as HTMLElement;
    expect(aside.className).toContain("panel-closing");
    expect(aside.getAttribute("aria-hidden")).toBe("true");
    expect(screen.queryByRole("complementary")).toBeNull();
    ends(aside, "panel-close");
    expect(closed).toHaveBeenCalled();
  });

  test("mounted again in strict mode, it still slides from the edge", () => {
    const { container } = render(<StrictMode>{panel()}</StrictMode>);
    const aside = container.querySelector("aside") as HTMLElement;
    expect(aside.className).toContain("panel-opening");
    expect(aside.style.getPropertyValue("--panel-from")).toBe("0px");
  });

  test("a panel replacing another starts from that one's width", () => {
    localStorage.setItem("botloft.panel.first", "480");
    const view = render(
      <SidePanel label="First" name="first" defaultWidth={320}>
        one
      </SidePanel>,
    );
    view.rerender(
      <SidePanel key="second" label="Second" name="second" defaultWidth={320}>
        two
      </SidePanel>,
    );
    const aside = view.container.querySelector("aside") as HTMLElement;
    expect(aside.style.getPropertyValue("--panel-from")).toBe("480px");
  });
});
