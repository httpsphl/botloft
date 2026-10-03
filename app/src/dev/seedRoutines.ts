// Dev only: routines for the preview, one of each kind of schedule and
// last run.

import type { FakeBotloft } from "../lib/fake";
import type { BotId, Routine, RoutineRun } from "../lib/protocol.gen";

const HOUR = 60 * 60_000;

export function seedRoutines(fake: FakeBotloft, bots: { scout: BotId; analyst: BotId }): void {
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  const create = (botId: BotId, fields: Partial<Routine> & Pick<Routine, "name" | "schedule">) =>
    fake.routines.handlers()["routines.create"]({
      botId,
      name: fields.name,
      prompt: fields.prompt ?? "",
      schedule: fields.schedule,
      timezone: zone,
    });
  const lastRun = (routine: Routine, run: Partial<RoutineRun>) => {
    routine.lastRun = {
      id: fake.id("rrn"),
      routineId: routine.id,
      scheduledFor: fake.now - 2 * HOUR,
      status: "done",
      reason: null,
      skippedCount: 0,
      messageId: null,
      createdAt: fake.now - 2 * HOUR,
      finishedAt: fake.now - HOUR,
      signal: null,
      ...run,
    };
  };

  const morning = create(bots.scout, {
    name: "Morning summary",
    prompt: "Summarize what arrived in shared/inbox and tell me what needs me.",
    schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "09:00" },
  });
  lastRun(morning, {});
  morning.nextRunAt = fake.now + 14 * HOUR;

  const links = create(bots.scout, {
    name: "Check the links",
    prompt: "Run the link checker on the report and fix what moved.",
    schedule: { kind: "interval", minutes: 120 },
  });
  lastRun(links, { status: "skipped", reason: "overlap", finishedAt: fake.now - HOUR });
  links.nextRunAt = fake.now + HOUR;

  const numbers = create(bots.analyst, {
    name: "Weekly numbers",
    prompt: "Recompute the growth numbers for the report.",
    schedule: { kind: "weekly", days: [1, 4], time: "14:30" },
  });
  lastRun(numbers, { status: "failed" });
  numbers.nextRunAt = fake.now + 3 * 24 * HOUR;

  const backup = create(bots.analyst, {
    name: "Nightly archive",
    prompt: "Zip shared/ into archive/.",
    schedule: { kind: "cron", expr: "0 2 * * *" },
  });
  backup.enabled = false;
  backup.nextRunAt = null;

  // Spec 20.13: runs when Analyst says the numbers are in.
  const recheck = create(bots.scout, {
    name: "Recheck the sources",
    prompt: "The numbers changed: check that every claim in the report still has a source.",
    schedule: { kind: "signal", name: "numbers-updated" },
  });
  lastRun(recheck, {
    signal: { name: "numbers-updated", fromBotId: bots.analyst, note: null },
  });
}
