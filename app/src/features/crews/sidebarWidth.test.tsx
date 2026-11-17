import { cleanup, fireEvent, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { SIDEBAR_DEFAULT_PX, SIDEBAR_MAX_PX, sidebarWidth } from "../../shell/sidebarWidth";
import { crewOpened, renderApp } from "../../test/app";

afterEach(() => {
  cleanup();
  sidebarWidth.reset();
});

const rootWidth = () => document.documentElement.style.getPropertyValue("--sidebar-width");

describe("the width of the crews list", () => {
  test("the arrow keys widen and narrow it, and a double click goes back", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");

    const handle = screen.getByRole("separator", { name: "Resize panel" });
    expect(handle.getAttribute("aria-valuenow")).toBe(String(SIDEBAR_DEFAULT_PX));

    fireEvent.keyDown(handle, { key: "ArrowRight" });
    expect(handle.getAttribute("aria-valuenow")).toBe(String(SIDEBAR_DEFAULT_PX + 24));
    expect(rootWidth()).toBe(`${SIDEBAR_DEFAULT_PX + 24}px`);
    expect(localStorage.getItem("botloft.panel.sidebar")).toBe(String(SIDEBAR_DEFAULT_PX + 24));

    fireEvent.keyDown(handle, { key: "ArrowLeft" });
    expect(handle.getAttribute("aria-valuenow")).toBe(String(SIDEBAR_DEFAULT_PX));

    fireEvent.doubleClick(handle);
    expect(handle.getAttribute("aria-valuenow")).toBe(String(SIDEBAR_DEFAULT_PX));
  });

  test("it never goes past its limits", () => {
    sidebarWidth.set(5000, true);
    expect(sidebarWidth.get()).toBe(SIDEBAR_MAX_PX);
    sidebarWidth.set(10, true);
    expect(sidebarWidth.get()).toBeGreaterThanOrEqual(200);
  });
});
