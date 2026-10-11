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
  test("the owner sits at the foot of the rail, even before a crew, with their plan", async () => {
    renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    const button = screen.getByRole("button", { name: "Ana Lima: account and settings" });
    expect(button.textContent).toBe("A");
    expect(button.title).toBe("Ana Lima · Max plan");
    fireEvent.click(button);
    const menu = screen.getByRole("menu", { name: "Ana Lima" });
    expect(menu.textContent).toContain("Max plan");
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

  test("usage of another agent comes under its own name, with its plan and its limit", async () => {
    const fake = new FakeBotloft();
    const now = Date.now();
    fake.system = {
      ...fake.system,
      usage: {
        status: "allowed",
        resetsAt: null,
        observedAt: now,
        windows: [{ name: "five_hour", utilization: 0.2, resetsAt: null }],
      },
      agentUsage: [
        {
          agent: "codex",
          plan: "free",
          usage: {
            status: "rejected",
            resetsAt: now + 3 * 86_400_000,
            observedAt: now,
            windows: [{ name: "thirty_day", utilization: 1, resetsAt: now + 3 * 86_400_000 }],
          },
        },
      ],
    };
    renderApp(fake);
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
    const dialog = screen.getByRole("dialog", { name: "Usage" });
    expect(within(dialog).getByRole("heading", { name: /^Claude Code/ })).toBeDefined();
    expect(within(dialog).getByRole("heading", { name: /^Codex/ })).toBeDefined();
    expect(dialog.textContent).toContain("Plan: free");
    expect(within(dialog).getByRole("progressbar", { name: "30-day window" })).toBeDefined();
    expect(dialog.textContent).toContain("100% used");
    expect(dialog.textContent).toContain("Agents on Codex wait until it resets.");
    // Claude Code is not limited: its own warning does not show.
    expect(dialog.textContent).not.toContain("Your agents wait until it resets.");
  });

  test("usage waits for an agent's first reply", async () => {
    renderApp();
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Usage" }));
    expect(screen.getByText("Usage shows up after an agent's first reply.")).toBeDefined();
  });

  test("settings change the theme and the language", async () => {
    renderApp();
    await openAccount();
    fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
    const dialog = screen.getByRole("region", { name: "Settings" });
    fireEvent.click(within(dialog).getByRole("tab", { name: "Appearance" }));
    fireEvent.click(within(dialog).getByRole("radio", { name: "Dark" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    fireEvent.click(within(dialog).getByRole("tab", { name: "General" }));
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Language: System language: English" }),
    );
    fireEvent.click(within(dialog).getByRole("option", { name: "Español" }));
    expect(screen.getByRole("region", { name: "Configuración" })).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Idioma: Español" }));
    fireEvent.click(screen.getByRole("option", { name: /Idioma del sistema/ }));
    fireEvent.click(screen.getByRole("tab", { name: "About" }));
    expect(screen.getByRole("region", { name: "Settings" }).textContent).toContain("Botloft 0.1.0");
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
