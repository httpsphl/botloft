import { act, cleanup } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import type { RoutineRun } from "../lib/protocol.gen";
import { crewOpened, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";
import { noticeOf } from "./routineAlerts";

afterEach(() => {
  cleanup();
  resetPrefs();
});

/** Scout's routine, which has failed `streak` times in a row so far. */
async function failing(streak = 0) {
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
  routine.failStreak = streak;
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  host.front = false;
  const finish = (status: "failed" | "done") => {
    const run: RoutineRun = {
      id: fake.id("rrn"),
      routineId: routine.id,
      scheduledFor: fake.now,
      status,
      reason: null,
      skippedCount: 0,
      nothingNew: false,
      waited: null,
      messageId: null,
      createdAt: fake.now,
      finishedAt: fake.now,
      signal: null,
    };
    act(() => fake.emit({ name: "routine.run", params: run }));
  };
  return { host, fail: () => finish("failed"), works: () => finish("done") };
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

describe("a routine that keeps failing", () => {
  test("rings once for the row and again when it works", () => {
    expect(noticeOf("failed", 0)).toBe("failed");
    expect(noticeOf("failed", 1)).toBeNull();
    expect(noticeOf("failed", 4)).toBeNull();
    expect(noticeOf("done", 3)).toBe("recovered");
    expect(noticeOf("done", 0)).toBeNull();
    expect(noticeOf("skipped", 2)).toBeNull();
  });

  test("tells nothing for the second failure in a row", async () => {
    const { host, fail } = await failing(1);
    fail();
    expect(host.notices).toEqual([]);
  });

  test("tells when it works again", async () => {
    const { host, works } = await failing(3);
    works();
    expect(host.notices).toEqual([
      { title: "Scout's routine works again", body: "Morning summary", sound: true },
    ]);
  });
});
