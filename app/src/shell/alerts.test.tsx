import { act, cleanup, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { SYSTEM } from "../lib/system";
import { crewOpened, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";

// These are the Windows notifications without the island, which takes
// their place when it is on (spec 31.6); its own tests are below.
beforeEach(() => {
  prefs.island.set(false);
});

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function scoutBehind() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  host.front = false;
  return { fake, host, scout, writer };
}

describe("notifications", () => {
  test("say which agent needs the owner and for what, with the Windows sound", async () => {
    const { fake, host, scout } = await scoutBehind();
    act(() => {
      fake.chat.ask(scout.id, "Bash", "npm test", '{"command":"npm test"}');
      fake.setBotState(scout.id, "needs_approval");
    });
    expect(host.notices).toEqual([
      { title: "Scout asks for your permission", body: "Run a command", sound: true },
    ]);
    act(() => fake.setBotState(scout.id, "auth_error"));
    expect(host.notices.at(-1)).toEqual({
      title: "Scout needs you to sign in to Claude",
      body: "Open Botloft to sign in.",
      sound: true,
    });
  });

  test("tell when an agent finishes only if the owner wants it, and stay quiet without sound", async () => {
    const { fake, host, scout } = await scoutBehind();
    act(() => fake.setBotState(scout.id, "busy"));
    act(() => fake.setBotState(scout.id, "idle"));
    expect(host.notices).toEqual([]);
    prefs.notifyDone.set(true);
    prefs.sound.set(false);
    act(() => fake.setBotState(scout.id, "busy"));
    act(() => fake.setBotState(scout.id, "idle"));
    expect(host.notices).toEqual([{ title: "Scout finished", body: "Crew Ops", sound: false }]);
  });

  test("do not show while Botloft is in front, nor for what was there on opening", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "needs_approval");
    const host = new FakeHost();
    renderApp(fake, host);
    await crewOpened("Ops");
    host.front = false;
    act(() => fake.setBotState(scout.id, "needs_approval"));
    expect(host.notices).toEqual([]);
    host.front = true;
    act(() => fake.setBotState(scout.id, "auth_error"));
    expect(host.notices).toEqual([]);
  });

  test("opening Botloft from one goes to its agent", async () => {
    const { fake, host, writer } = await scoutBehind();
    act(() => fake.setBotState(writer.id, "needs_approval"));
    expect(host.notices).toHaveLength(1);
    act(() => host.reopen());
    await waitFor(() =>
      expect(screen.getByRole("heading", { level: 1, name: "Writer" })).toBeDefined(),
    );
  });
});

describe.runIf(SYSTEM === "Windows")("with the island on", () => {
  test("no notification while the screen is free, and the usual one when it is quiet", async () => {
    const { fake, host, scout } = await scoutBehind();
    prefs.island.set(true);
    act(() => fake.setBotState(scout.id, "needs_approval"));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(host.notices).toEqual([]);

    host.quietNow = true;
    act(() => fake.setBotState(scout.id, "auth_error"));
    await waitFor(() => expect(host.notices).toHaveLength(1));
  });
});
