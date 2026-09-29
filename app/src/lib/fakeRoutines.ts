// The fake daemon's `routines.*` methods (spec 20.8): routines kept in
// memory, with "run now" sending the routine's message to the bot. The
// schedule itself is the daemon's business and is not played here.

import type { FakeBotloft, Handlers } from "./fake";
import { conflict, invalid, notFound } from "./fakeRules";
import type { Routine, RoutineId, RoutineRun } from "./protocol.gen";

type RoutineMethods = Extract<keyof Handlers, `routines.${string}`>;

export class FakeRoutines {
  readonly routines = new Map<RoutineId, Routine>();
  readonly runs: RoutineRun[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  handlers(): Pick<Handlers, RoutineMethods> {
    return {
      "routines.list": ({ botId }) =>
        [...this.routines.values()].filter(
          (routine) =>
            routine.archivedAt === null && (botId === undefined || routine.botId === botId),
        ),
      "routines.create": ({ botId, name, prompt, schedule, timezone, overlap, missed }) => {
        this.fake.bot(botId);
        if (!name.trim()) {
          throw invalid("name must not be empty");
        }
        const routine: Routine = {
          id: this.fake.id("rtn"),
          botId,
          name: name.trim(),
          prompt,
          schedule,
          timezone,
          overlap: overlap ?? "skip",
          missed: missed ?? "run_once",
          enabled: true,
          nextRunAt: this.fake.now + 60 * 60_000,
          lastRun: null,
          createdAt: this.fake.now,
          updatedAt: this.fake.now,
          archivedAt: null,
        };
        this.routines.set(routine.id, routine);
        return this.changed(routine);
      },
      "routines.update": ({ routineId, ...changes }) => {
        const routine = this.routine(routineId);
        for (const [key, value] of Object.entries(changes)) {
          if (value !== undefined) {
            Object.assign(routine, { [key]: value });
          }
        }
        routine.updatedAt = this.fake.now;
        return this.changed(routine);
      },
      "routines.setEnabled": ({ routineId, enabled }) => {
        const routine = this.routine(routineId);
        routine.enabled = enabled;
        routine.nextRunAt = enabled ? this.fake.now + 60 * 60_000 : null;
        return this.changed(routine);
      },
      "routines.runNow": ({ routineId }) => {
        const routine = this.routine(routineId);
        const { message } = this.fake.conversation.say({
          to: routine.botId,
          body: routine.prompt,
          kind: "routine",
        });
        message.routineId = routine.id;
        const run: RoutineRun = {
          id: this.fake.id("rrn"),
          routineId,
          scheduledFor: this.fake.now,
          status: "queued",
          reason: null,
          skippedCount: 0,
          messageId: message.id,
          createdAt: this.fake.now,
          finishedAt: null,
        };
        this.runs.push(run);
        routine.lastRun = run;
        this.fake.emit({ name: "routine.run", params: run });
        this.changed(routine);
        return run;
      },
      "routines.archive": ({ routineId }) => {
        const routine = this.routine(routineId, false);
        if (routine.archivedAt === null) {
          routine.archivedAt = this.fake.now;
          routine.nextRunAt = null;
          this.changed(routine);
        }
        return routine;
      },
      "routines.runs": ({ routineId, limit }) => {
        this.routine(routineId, false);
        return this.runs
          .filter((run) => run.routineId === routineId)
          .reverse()
          .slice(0, limit ?? 50);
      },
    };
  }

  private routine(id: RoutineId, active = true): Routine {
    const routine = this.routines.get(id);
    if (!routine) {
      throw notFound(`routine ${id}`);
    }
    if (active && routine.archivedAt !== null) {
      throw conflict(`routine ${id} is archived`);
    }
    return routine;
  }

  private changed(routine: Routine): Routine {
    this.fake.emit({ name: "routine.changed", params: routine });
    return routine;
  }
}
