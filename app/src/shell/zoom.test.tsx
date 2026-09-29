import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../App";
import type { Client } from "../lib/client";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { currentZoom, DEFAULT_ZOOM, setZoom, stepZoom } from "./zoom";

afterEach(() => {
  cleanup();
  setZoom(DEFAULT_ZOOM);
});

function renderApp() {
  const host = new FakeHost();
  render(<App host={host} connect={() => new FakeBotloft() as Client} />);
  return host;
}

const key = (options: KeyboardEventInit) => fireEvent.keyDown(window, options);

describe("app size", () => {
  test("the app opens at the default size, bigger than the compact one", async () => {
    const host = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    expect(DEFAULT_ZOOM).toBe(1.25);
    expect(host.zoom).toBe(1.25);
  });

  test("the size menu sets the size and remembers it", async () => {
    const host = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    fireEvent.click(screen.getByRole("button", { name: "Size" }));
    expect(
      screen.getByRole("menuitemradio", { name: "125% (default)" }).getAttribute("aria-checked"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("menuitemradio", { name: "150%" }));
    await waitFor(() => expect(host.zoom).toBe(1.5));
    expect(localStorage.getItem("botloft.zoom")).toBe("1.5");
  });

  test("Ctrl+= and Ctrl+- step through the sizes and Ctrl+0 goes back", async () => {
    const host = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    key({ key: "=", ctrlKey: true });
    await waitFor(() => expect(host.zoom).toBe(1.5));
    key({ key: "-", ctrlKey: true });
    key({ key: "-", ctrlKey: true });
    key({ key: "-", ctrlKey: true });
    await waitFor(() => expect(host.zoom).toBe(1));
    key({ key: "0", ctrlKey: true });
    await waitFor(() => expect(host.zoom).toBe(1.25));
    key({ key: "=" });
    expect(currentZoom()).toBe(1.25);
  });

  test("the size stays within the levels", () => {
    setZoom(1.5);
    stepZoom(1);
    expect(currentZoom()).toBe(1.5);
    setZoom(1);
    stepZoom(-1);
    expect(currentZoom()).toBe(1);
  });
});
