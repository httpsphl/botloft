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
