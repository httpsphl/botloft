import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost, FakeUpdate } from "../../lib/fakeHost";
import { RpcError } from "../../lib/rpc";
import { prefs, resetPrefs } from "../../shell/prefs";

afterEach(() => {
  cleanup();
  resetPrefs();
});

function renderApp(fake = new FakeBotloft(), host = new FakeHost()) {
  render(<App host={host} connect={() => fake as Client} />);
  return { fake, host };
}

async function openSettings() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  return screen.getByRole("dialog", { name: "Settings" });
}

describe("settings", () => {
  test("in the background: each switch says what it does and is saved", async () => {
    const { fake } = renderApp();
    const dialog = await openSettings();
    const keep = within(dialog).getByRole("switch", {
      name: "Keep working after you close Botloft",
    });
    const start = within(dialog).getByRole("switch", { name: "Start with Windows" });
    const awake = within(dialog).getByRole("switch", {
      name: "Keep the computer awake while bots work",
    });
    await waitFor(() => expect(start.hasAttribute("disabled")).toBe(false));
    expect([keep, start, awake].map((s) => s.getAttribute("aria-checked"))).toEqual([
      "true",
      "true",
      "true",
    ]);
    expect(dialog.textContent).toContain("Your bots go on working and answering");

    fireEvent.click(keep);
    expect(keep.getAttribute("aria-checked")).toBe("false");
    expect(dialog.textContent).toContain("Closing Botloft stops every bot.");
    expect(localStorage.getItem("botloft.whenClosed")).toBe("stop");

    fireEvent.click(start);
    expect(start.getAttribute("aria-checked")).toBe("false");
    expect(dialog.textContent).toContain("the bots wait until you open Botloft");
    fireEvent.click(awake);
    await waitFor(() =>
      expect(fake.settings).toEqual({ startWithWindows: false, keepAwake: false }),
    );
    expect(
      fake.calls.filter((call) => call.method === "settings.update").map((c) => c.params),
    ).toEqual([{ startWithWindows: false }, { keepAwake: false }]);
  });

  test("a setting that cannot be saved goes back", async () => {
    const { fake } = renderApp();
    const dialog = await openSettings();
    const start = within(dialog).getByRole("switch", { name: "Start with Windows" });
    await waitFor(() => expect(start.hasAttribute("disabled")).toBe(false));
    fake.failNext("settings.update", new RpcError(-32603, "could not save the settings"));
    fireEvent.click(start);
    await screen.findByText(/Could not change the setting/);
    expect(start.getAttribute("aria-checked")).toBe("true");
    expect(fake.settings.startWithWindows).toBe(true);
  });

  test("about looks for a new version on request", async () => {
    const { host } = renderApp();
    const dialog = await openSettings();
    fireEvent.click(within(dialog).getByRole("tab", { name: "About" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Check for updates" }));
    expect(await within(dialog).findByText("You have the latest version.")).toBeDefined();
    host.update = new FakeUpdate("9.9.9");
    fireEvent.click(within(dialog).getByRole("button", { name: "Check for updates" }));
    expect(await within(dialog).findByText("Botloft 9.9.9 is out.")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "See the update" }));
    expect(screen.getByRole("dialog", { name: "Update Botloft" }).textContent).toContain("9.9.9");
  });
});

describe("closing the window", () => {
  test("leaves the bots working unless the owner chose otherwise", async () => {
    const { host } = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    await act(() => host.requestClose());
    expect(host.closed).toBe(true);
    expect(host.stops).toBe(0);
  });

  test("stops the bots, with the window out of sight first, when chosen", async () => {
    const { host } = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    prefs.whenClosed.set("stop");
    await act(() => host.requestClose());
    expect(host.hidden).toBe(true);
    expect(host.stops).toBe(1);
    expect(host.closed).toBe(true);
  });

  test("opening Botloft again starts the bots it stopped", async () => {
    prefs.whenClosed.set("stop");
    const host = new FakeHost();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    let finish: () => void = () => {};
    const installed = host.afterInstall;
    host.installDaemon = () =>
      new Promise((resolve) => {
        finish = () => resolve(installed as never);
      });
    renderApp(new FakeBotloft(), host);
    expect(await screen.findByText("Starting your bots…")).toBeDefined();
    expect(screen.queryByText(/In Settings you choose/)).toBeNull();
    await act(async () => finish());
    expect(await screen.findByText(/Running while this window is open/)).toBeDefined();
  });

  test("the welcome screen says Botloft waits for the owner after a restart", async () => {
    const fake = new FakeBotloft();
    fake.settings = { startWithWindows: false, keepAwake: true };
    renderApp(fake);
    expect(
      await screen.findByText(
        "Running in the background, so your bots keep working after you close this window.",
      ),
    ).toBeDefined();
  });
});
