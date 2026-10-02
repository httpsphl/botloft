// Routines for the design system's sample data (spec 20): schedules as the
// app writes them and routines in the protocol's shape, for CrewRoutines.

import type { Bot, Routine, RunStatus, Schedule, SkipReason } from "../../app/src/lib/protocol.gen";
import { ago, id } from "./ids";

export const zone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;

/** Schedules as the app writes them: days are 1 = Monday ... 7 = Sunday, time "HH:MM". */
export const schedule = {
  weekly(days: number[], time: string): Schedule {
    return { kind: "weekly", days, time };
  },
  weekdays(time: string): Schedule {
    return { kind: "weekly", days: [1, 2, 3, 4, 5], time };
  },
  every(minutes: number): Schedule {
    return { kind: "interval", minutes };
  },
  cron(expr: string): Schedule {
    return { kind: "cron", expr };
  },
};

export interface RoutineSeed {
  bot: Bot;
  name: string;
  /** The message the bot gets each time it runs. */
  prompt?: string;
  schedule: Schedule;
  enabled?: boolean;
  /**
   * When it runs next, from now; null for never. Left out, it follows the
   * schedule: the next of its days and time, or one interval after the last run.
   */
  nextInMinutes?: number | null;
  /** How it went last time; "queued" reads as running now. */
  lastRun?: { status: RunStatus; minutesAgo?: number; reason?: SkipReason; skippedCount?: number };
}

/** The next run by the schedule, in local time; `lastAt` anchors an interval. */
function nextBySchedule(rule: Schedule, lastAt: number | null): number | null {
  const now = Date.now();
  switch (rule.kind) {
    case "interval": {
      const step = rule.minutes * 60_000;
      let next = (lastAt ?? now) + step;
      while (next <= now) next += step;
      return next;
    }
    case "weekly": {
      const [hours = 0, minutes = 0] = rule.time.split(":").map(Number);
      for (let ahead = 0; ahead <= 7; ahead++) {
        const day = new Date(now);
        day.setDate(day.getDate() + ahead);
        day.setHours(hours, minutes, 0, 0);
        // getDay(): 0 = Sunday; the schedule's days: 1 = Monday ... 7 = Sunday.
        const weekday = day.getDay() === 0 ? 7 : day.getDay();
        if (rule.days.includes(weekday) && day.getTime() > now) {
          return day.getTime();
        }
      }
      return null;
    }
    case "cron":
      // Not worked out here: an hour from now.
      return now + 60 * 60_000;
  }
}

export function makeRoutine(seed: RoutineSeed): Routine {
  const routineId = id("rtn");
  const last = seed.lastRun;
  const lastAt = ago(last?.minutesAgo ?? 0);
  return {
    id: routineId,
    botId: seed.bot.id,
    name: seed.name,
    prompt: seed.prompt ?? "",
    schedule: seed.schedule,
    timezone: zone(),
    overlap: "skip",
    missed: "run_once",
    enabled: seed.enabled ?? true,
    nextRunAt:
      seed.nextInMinutes === null
        ? null
        : seed.nextInMinutes !== undefined
          ? Date.now() + seed.nextInMinutes * 60_000
          : nextBySchedule(seed.schedule, last ? lastAt : null),
    lastRun: last
      ? {
          id: id("run"),
          routineId,
          scheduledFor: lastAt,
          status: last.status,
          reason: last.reason ?? null,
          skippedCount: last.skippedCount ?? 0,
          messageId: null,
          createdAt: lastAt,
          finishedAt: last.status === "queued" ? null : lastAt,
        }
      : null,
    createdAt: ago(60 * 24 * 2),
    updatedAt: lastAt,
    archivedAt: null,
  };
}

