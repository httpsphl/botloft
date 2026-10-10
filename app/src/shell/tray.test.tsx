import { act, cleanup, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { crewOpened, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";

afterEach(() => {
  cleanup();
  resetPrefs();
});

function crews() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const docs = fake.addCrew("Docs");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, ops, docs, scout };
}

const view = (host: FakeHost) => host.tray?.view;

describe("the icon near the clock", () => {
  test("says what the agents are doing, with a dot when something waits", async () => {
    const { fake, scout } = crews();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    await waitFor(() => expect(view(host)?.status).toBe("No agent working"));
    expect(view(host)).toMatchObject({
      tooltip: "Botloft: No agent working",
      attention: false,
      open: "Open Botloft",
      pause: "Pause every crew",
      quit: "Quit Botloft",
    });
    act(() => fake.setBotState(scout.id, "busy"));
    expect(view(host)?.status).toBe("1 agent working");
    act(() => fake.setBotState(scout.id, "needs_approval"));
    expect(view(host)).toMatchObject({ status: "Scout needs you", attention: true });
  });

  test("opens the window, pauses and resumes every crew, and quits", async () => {
    const { fake, ops, docs } = crews();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    await waitFor(() => expect(host.tray).not.toBeNull());

    await act(() => host.requestClose());
    expect(host.hidden).toBe(true);
    expect(host.closed).toBe(false);
    act(() => host.tray?.actions.open());
    expect(host.hidden).toBe(false);

    await act(async () => host.tray?.actions.pause());
    await waitFor(() => expect(view(host)?.pause).toBe("Resume every crew"));
    expect([fake.crew(ops.id).paused, fake.crew(docs.id).paused]).toEqual([true, true]);
    await act(async () => host.tray?.actions.pause());
    await waitFor(() => expect(view(host)?.pause).toBe("Pause every crew"));
    expect([fake.crew(ops.id).paused, fake.crew(docs.id).paused]).toEqual([false, false]);

    act(() => host.tray?.actions.quit());
    expect(host.closed).toBe(true);
    expect(host.stops).toBe(0);
  });

  test("is not there when the owner turns it off, and closing then closes Botloft", async () => {
    const { fake } = crews();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    await waitFor(() => expect(host.tray).not.toBeNull());
    act(() => prefs.tray.set(false));
    expect(host.tray).toBeNull();
    await act(() => host.requestClose());
    expect(host.closed).toBe(true);
    expect(host.stops).toBe(0);
  });
});

describe("opening at sign-in", () => {
  test("follows starting with Windows, the icon and the window choice", async () => {
    const { fake } = crews();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    await waitFor(() => expect(host.openAtSignIn).toBe(true));
    act(() => prefs.tray.set(false));
    expect(host.openAtSignIn).toBe(false);
    act(() => prefs.openAtSignIn.set(true));
    expect(host.openAtSignIn).toBe(true);
  });

  test("does not open the app when the agents do not start with Windows", async () => {
    const { fake } = crews();
    fake.settings = { ...fake.settings, startWithWindows: false };
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    await waitFor(() => expect(host.openAtSignIn).toBe(false));
  });

  test("leaves the window hidden near the clock, unless the owner wants it", async () => {
    const { fake } = crews();
    const host = new FakeHost();
    host.atSignIn = true;
    host.hidden = true;
    renderApp(fake, host);
    await crewOpened("Ops");
    await waitFor(() => expect(host.tray).not.toBeNull());
    expect(host.hidden).toBe(true);
    cleanup();

    prefs.openAtSignIn.set(true);
    const again = new FakeHost();
    again.atSignIn = true;
    again.hidden = true;
    renderApp(crews().fake, again);
    await waitFor(() => expect(again.hidden).toBe(false));
  });

  test("shows the window when Botloft cannot start", async () => {
    const host = new FakeHost();
    host.atSignIn = true;
    host.hidden = true;
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    host.afterInstall = new Error("no way");
    renderApp(new FakeBotloft(), host);
    await screen.findByRole("alert");
    expect(host.hidden).toBe(false);
  });
});
