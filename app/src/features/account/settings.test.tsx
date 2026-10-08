import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost, FakeUpdate } from "../../lib/fakeHost";
import { RpcError } from "../../lib/rpc";
import { SYSTEM } from "../../lib/system";
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
  return screen.getByRole("region", { name: "Settings" });
}

describe("settings", () => {
  test("in the background: each switch says what it does and is saved", async () => {
    const { fake } = renderApp();
    const dialog = await openSettings();
    const keep = within(dialog).getByRole("switch", {
      name: "Keep working after you close Botloft",
    });
    const start = within(dialog).getByRole("switch", { name: `Start with ${SYSTEM}` });
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
      expect(fake.settings).toEqual({
        startWithWindows: false,
        keepAwake: false,
        approvalWaitMinutes: 60,
      }),
    );
    expect(
      fake.calls.filter((call) => call.method === "settings.update").map((c) => c.params),
    ).toEqual([{ startWithWindows: false }, { keepAwake: false }]);
  });

  test("the icon and the window at sign-in show only where they apply", async () => {
    renderApp();
    const dialog = await openSettings();
    const icon = () =>
      within(dialog).queryByRole("switch", { name: "Show Botloft near the clock" });
    const atSignIn = () =>
      within(dialog).queryByRole("switch", {
        name: `Open the window when you sign in to ${SYSTEM}`,
      });
    await waitFor(() => expect(atSignIn()).not.toBeNull());
    expect(dialog.textContent).toContain(
      "Botloft starts near the clock, without opening the window.",
    );
    fireEvent.click(icon() as HTMLElement);
    expect(dialog.textContent).toContain("Closing the window closes Botloft.");
    expect(dialog.textContent).toContain("The window opens only when you open Botloft.");
    fireEvent.click(
      within(dialog).getByRole("switch", { name: "Keep working after you close Botloft" }),
    );
    expect(icon()).toBeNull();
    fireEvent.click(within(dialog).getByRole("switch", { name: `Start with ${SYSTEM}` }));
    await waitFor(() => expect(atSignIn()).toBeNull());
  });

  test("notifications: when, and with a sound", async () => {
    renderApp();
    const dialog = await openSettings();
    fireEvent.click(within(dialog).getByRole("tab", { name: "Notifications" }));
    const on = (name: string) =>
      within(dialog).getByRole("switch", { name }).getAttribute("aria-checked");
    expect([on("When a bot needs you"), on("When a bot finishes"), on("Play a sound")]).toEqual([
      "true",
      "false",
      "true",
    ]);
    fireEvent.click(within(dialog).getByRole("switch", { name: "When a bot finishes" }));
    expect(prefs.notifyDone.get()).toBe(true);
    expect(dialog.textContent).not.toContain("notifications only come");
    act(() => prefs.tray.set(false));
    expect(dialog.textContent).toContain(
      "With the window closed, notifications only come with Botloft near the clock",
    );
  });

  test("a setting that cannot be saved goes back", async () => {
    const { fake } = renderApp();
    const dialog = await openSettings();
    const start = within(dialog).getByRole("switch", { name: `Start with ${SYSTEM}` });
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
  test("leaves the bots working: near the clock, or closed without the icon", async () => {
    const { host } = renderApp();
    await screen.findByRole("heading", { name: "Welcome to Botloft" });
    await waitFor(() => expect(host.tray).not.toBeNull());
    await act(() => host.requestClose());
    expect([host.hidden, host.closed, host.stops]).toEqual([true, false, 0]);
    act(() => prefs.tray.set(false));
    await act(() => host.requestClose());
    expect([host.closed, host.stops]).toEqual([true, 0]);
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
    fake.settings = { ...fake.settings, startWithWindows: false };
    renderApp(fake);
    expect(
      await screen.findByText(
        "Running in the background, so your bots keep working after you close this window.",
      ),
    ).toBeDefined();
  });
});

describe("the page of settings", () => {
  test("has its parts in groups, and the account is not in Backup", async () => {
    renderApp();
    const page = await openSettings();
    for (const group of ["Every day", "Account and connections", "Your data", "Botloft"]) {
      expect(within(page).getByText(group)).toBeDefined();
    }
    const tabs = within(page)
      .getAllByRole("tab")
      .map((tab) => tab.textContent);
    expect(tabs).toEqual([
      "General",
      "Appearance",
      "Chat",
      "Notifications",
      "Account and phone",
      "Connected tools",
      "Backup",
      "Archived",
      "About",
    ]);

    fireEvent.click(within(page).getByRole("tab", { name: "Backup" }));
    expect(within(page).queryByLabelText("Your e-mail")).toBeNull();
    expect(await within(page).findByText("Copies in the cloud need your account.")).toBeDefined();
    fireEvent.click(within(page).getByRole("button", { name: "Go to Account and phone" }));
    expect(await within(page).findByLabelText("Your e-mail")).toBeDefined();
    expect(
      within(page).getByRole("tab", { name: "Account and phone" }).getAttribute("aria-selected"),
    ).toBe("true");
  });

  test("a search finds a setting by its name and takes the owner to its page", async () => {
    renderApp();
    const page = await openSettings();
    const box = within(page).getByRole("searchbox", { name: "Search settings" });
    fireEvent.change(box, { target: { value: "THEME" } });
    const results = within(page).getByRole("list", { name: "Search settings" });
    expect(results.textContent).toContain("In Appearance");
    fireEvent.click(within(results).getByRole("button", { name: /Theme/ }));
    expect(
      within(page).getByRole("tab", { name: "Appearance" }).getAttribute("aria-selected"),
    ).toBe("true");
    expect((box as HTMLInputElement).value).toBe("");

    fireEvent.change(box, { target: { value: "zzzz" } });
    expect(within(page).getByText('Nothing in Settings matches "zzzz".')).toBeDefined();
  });

  test("opens before the first crew too, where a copy can be brought back", async () => {
    renderApp(new FakeBotloft());
    const page = await openSettings();
    expect(within(page).getByRole("tab", { name: "Backup" })).toBeDefined();
  });
});
