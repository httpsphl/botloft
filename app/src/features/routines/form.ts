// The routine form's fields and the schedule they stand for (spec 20.9):
// every day, weekdays, chosen days or every N minutes or hours, or a cron
// expression under "More options".

import type { Missed, Overlap, Routine, Schedule } from "../../lib/protocol.gen";
import { systemZone } from "./describe";

export type Frequency = "daily" | "weekdays" | "days" | "interval";
export type Unit = "minutes" | "hours";

export interface RoutineForm {
  name: string;
  prompt: string;
  frequency: Frequency;
  /** 1 = Monday … 7 = Sunday, for chosen days. */
  days: number[];
  /** `HH:MM`. */
  time: string;
  every: number;
  unit: Unit;
  timezone: string;
  overlap: Overlap;
  missed: Missed;
  useCron: boolean;
  cron: string;
}

const EVERY_DAY = [1, 2, 3, 4, 5, 6, 7];
const WEEKDAYS = [1, 2, 3, 4, 5];

export function blankForm(): RoutineForm {
  return {
    name: "",
    prompt: "",
    frequency: "weekdays",
    days: [1],
    time: "09:00",
    every: 1,
    unit: "hours",
    timezone: systemZone(),
    overlap: "skip",
    missed: "run_once",
    useCron: false,
    cron: "",
  };
}

/** The form as it opens for an existing routine. */
export function formOf(routine: Routine): RoutineForm {
  const form: RoutineForm = {
    ...blankForm(),
    name: routine.name,
    prompt: routine.prompt,
    timezone: routine.timezone,
    overlap: routine.overlap,
    missed: routine.missed,
  };
  const schedule = routine.schedule;
  switch (schedule.kind) {
    case "weekly": {
      const days = [...new Set(schedule.days)].sort((a, b) => a - b);
      const key = days.join(",");
      form.time = schedule.time;
      form.days = days;
      form.frequency =
        key === EVERY_DAY.join(",") ? "daily" : key === WEEKDAYS.join(",") ? "weekdays" : "days";
      break;
    }
    case "interval":
      form.frequency = "interval";
      if (schedule.minutes % 60 === 0) {
        form.every = schedule.minutes / 60;
        form.unit = "hours";
      } else {
        form.every = schedule.minutes;
        form.unit = "minutes";
      }
      break;
    case "cron":
      form.useCron = true;
      form.cron = schedule.expr;
      break;
  }
  return form;
}

/** The schedule the form describes. */
export function scheduleOf(form: RoutineForm): Schedule {
  if (form.useCron) {
    return { kind: "cron", expr: form.cron.trim() };
  }
  switch (form.frequency) {
    case "daily":
      return { kind: "weekly", days: EVERY_DAY, time: form.time };
    case "weekdays":
      return { kind: "weekly", days: WEEKDAYS, time: form.time };
    case "days":
      return { kind: "weekly", days: [...form.days].sort((a, b) => a - b), time: form.time };
    case "interval":
      return {
        kind: "interval",
        minutes: Math.round(form.every) * (form.unit === "hours" ? 60 : 1),
      };
  }
}
