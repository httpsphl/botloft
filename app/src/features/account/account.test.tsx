import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { setTheme } from "../../shell/theme";

afterEach(() => {
  cleanup();
  setTheme("system");
});

function renderApp(fake = new FakeBotloft(), host = new FakeHost()) {
  render(<App host={host} connect={() => fake as Client} />);
  return { fake, host };
}

async function openAccount() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
}

describe("account area", () => {
  test("the owner and their plan show at the bottom of the sidebar, even before a crew", async () => {
    renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    const button = screen.getByRole("button", { name: "Ana Lima: account and settings" });
    expect(button.textContent).toContain("Ana Lima");
    expect(button.textContent).toContain("Max plan");
  });

  test("the title bar keeps no settings once connected", async () => {
    renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    expect(screen.queryByRole("button", { name: "Language" })).toBeNull();
    expect(screen.queryByRole("button", { name: /Theme/ })).toBeNull();
    expect(screen.queryByRole("button", { name: "Size" })).toBeNull();
  });

  test("the menu shows the email and opens the pages", async () => {
    const { host } = renderApp();
    await openAccount();
    const menu = screen.getByRole("menu", { name: "Ana Lima" });
    expect(menu.textContent).toContain("ana@example.com");
    fireEvent.click(within(menu).getByRole("menuitem", { name: "What's new" }));
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Help" }));
    await waitFor(() =>
      expect(host.opened).toEqual([
        "https://github.com/httpsphl/botloft/releases",
        "https://github.com/httpsphl/botloft#readme",
      ]),
    );
  });

  test("usage shows each window, how much is used and when it resets", async () => {
    const fake = new FakeBotloft();
    const now = Date.now();
    fake.system = {
      ...fake.system,
      usage: {
        status: "allowed",
        resetsAt: now + 2 * 3600_000,
        observedAt: now,
        windows: [
          { name: "five_hour", utilization: 0.34, resetsAt: now + 2 * 3600_000 },
          { name: "seven_day", utilization: 0.915, resetsAt: null },
        ],
      },
    };
    renderApp(fake);
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
    const dialog = screen.getByRole("dialog", { name: "Usage" });
    expect(within(dialog).getByRole("progressbar", { name: "5-hour window" })).toBeDefined();
    expect(dialog.textContent).toContain("34% used");
    expect(dialog.textContent).toContain("Resets in 2 hr.");
    expect(dialog.textContent).toContain("92% used");
  });

  test("usage waits for a bot's first reply", async () => {
    renderApp();
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
    expect(screen.getByText("Usage shows up after a bot's first reply.")).toBeDefined();
  });

  test("settings change the theme and the language", async () => {
    renderApp();
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
    const dialog = screen.getByRole("dialog", { name: "Settings" });
    fireEvent.click(within(dialog).getByRole("tab", { name: "Appearance" }));
    fireEvent.click(within(dialog).getByRole("radio", { name: "Dark" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    fireEvent.click(within(dialog).getByRole("tab", { name: "General" }));
    fireEvent.click(within(dialog).getByRole("radio", { name: "Español" }));
    expect(screen.getByRole("dialog", { name: "Configuración" })).toBeDefined();
    fireEvent.click(screen.getByRole("radio", { name: /Idioma del sistema/ }));
    fireEvent.click(screen.getByRole("tab", { name: "About" }));
    expect(screen.getByRole("dialog", { name: "Settings" }).textContent).toContain("Botloft 0.1.0");
  });

  test("the setup screens keep the language menu, since they have no sidebar", async () => {
    const host = new FakeHost();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    host.afterInstall = new Error("no way");
    renderApp(new FakeBotloft(), host);
    await screen.findByRole("alert");
    expect(screen.getByRole("button", { name: "Language" })).toBeDefined();
  });
});
