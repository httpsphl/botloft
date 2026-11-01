// The phone's Home (spec 28.15): the owner's shortcuts, which run with a tap,
// and what the crews did since this phone's midnight. In memory only, like
// the conversations.

import type { DayActivity, FromPhone, PhoneShortcut, ToPhone } from "../lib/protocol.gen";
import type { PhoneState } from "./client";

/** How a shortcut just tapped is doing. */
export type Run = "running" | "started" | "failed";

/** The part of the phone's state that is about the Home. */
export interface HomePart {
  shortcuts: PhoneShortcut[];
  /** The day so far; null before the first answer. */
  today: DayActivity | null;
  homeLoaded: boolean;
  /** The shortcuts just tapped, by routine, until the word about them goes. */
  runs: Record<string, Run>;
}

export const noHome: HomePart = { shortcuts: [], today: null, homeLoaded: false, runs: {} };

/** The start of the day where this phone is, in Unix ms. */
export function midnight(now = Date.now()): number {
  const day = new Date(now);
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

export class HomeTalk {
  private counter = 0;
  /** The routine each `routine.run` on its way is for, by its id. */
  private readonly asked = new Map<string, string>();
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(
    private readonly get: () => PhoneState,
    private readonly set: (change: Partial<PhoneState>) => void,
    private readonly send: (message: FromPhone) => Promise<boolean>,
    /** How long "started" or "did not run" stays on a shortcut. */
    private readonly wordFor = 4000,
    /** How long a run may wait for the computer's answer. */
    private readonly runWait = 20000,
  ) {}

  /** Asks for the shortcuts and the day so far. */
  ask = async (): Promise<void> => {
    await this.send({ t: "home", since: midnight() });
  };

  /** Runs a shortcut; false when it could not even be sent. */
  run = async (routineId: string): Promise<boolean> => {
    if (this.get().runs[routineId] === "running") {
      return false;
    }
    this.counter += 1;
    const clientId = `${Date.now().toString(36)}-r${this.counter}`;
    this.asked.set(clientId, routineId);
    this.mark(routineId, "running");
    const sent = await this.send({ t: "routine.run", clientId, routineId }).catch(() => false);
    if (!sent) {
      this.asked.delete(clientId);
      this.mark(routineId, "failed");
      return false;
    }
    this.later(routineId, this.runWait, () => {
      if (this.asked.delete(clientId)) {
        this.mark(routineId, "failed");
      }
    });
    return true;
  };

  /** Whether `message` was about the Home. */
  handle(message: ToPhone): boolean {
    switch (message.t) {
      case "home":
        this.set({ shortcuts: message.shortcuts, today: message.today ?? null, homeLoaded: true });
        return true;
      case "ran": {
        const routineId = this.asked.get(message.clientId);
        if (routineId) {
          this.asked.delete(message.clientId);
          this.mark(routineId, message.ok ? "started" : "failed");
          if (message.ok) {
            void this.ask();
          }
        }
        return true;
      }
      default:
        return false;
    }
  }

  private later(routineId: string, ms: number, then: () => void): void {
    clearTimeout(this.timers.get(routineId));
    this.timers.set(routineId, setTimeout(then, ms));
  }

  /** Says how the shortcut is doing; a word that is not "running" goes after a while. */
  private mark(routineId: string, run: Run): void {
    clearTimeout(this.timers.get(routineId));
    this.timers.delete(routineId);
    this.set({ runs: { ...this.get().runs, [routineId]: run } });
    if (run !== "running") {
      this.later(routineId, this.wordFor, () => {
        const runs = { ...this.get().runs };
        delete runs[routineId];
        this.timers.delete(routineId);
        this.set({ runs });
      });
    }
  }
}
