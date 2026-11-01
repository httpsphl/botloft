import { act, cleanup } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import type { RoutineRun } from "../lib/protocol.gen";
import { crewOpened, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function failing() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  const routine = fake.routines.handlers()["routines.create"]({
    botId: scout.id,
    name: "Morning summary",
    prompt: "Summarize",
    schedule: { kind: "weekly", days: [1], time: "09:00" },
    timezone: "UTC",
  });
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  host.front = false;
  const fail = () => {
    const run: RoutineRun = {
      id: fake.id("rrn"),
      routineId: routine.id,
      scheduledFor: fake.now,
      status: "failed",
      reason: null,
      skippedCount: 0,
      messageId: null,
      createdAt: fake.now,
      finishedAt: fake.now,
      signal: null,
    };
    act(() => fake.emit({ name: "routine.run", params: run }));
  };
  return { host, fail };
}

describe("a routine that fails", () => {
  test("is told in a Windows notification with the agent and the routine", async () => {
    const { host, fail } = await failing();
    fail();
    expect(host.notices).toEqual([
      { title: "Scout's routine failed", body: "Morning summary", sound: true },
    ]);
  });

  test("tells nothing when the owner turned it off", async () => {
    const { host, fail } = await failing();
    prefs.notifyRoutineFailed.set(false);
    fail();
    expect(host.notices).toEqual([]);
  });
});
