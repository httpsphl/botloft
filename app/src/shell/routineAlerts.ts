// Telling the owner when a routine fails, once per row of failures, and
// when it works again (spec 20.9, 15.1): a Windows notification with the
// agent and the routine, or the chime while Botloft is in front, and that
// agent's chat opened when the owner comes back from the notification.

import { useEffect } from "react";
import { t } from "../i18n";
import type { BotId, RunStatus } from "../lib/protocol.gen";
import { useApi, useAppStore, useHost } from "../store/context";
import { prefs } from "./prefs";
import { playSound } from "./sounds";

/** How long after a notification opening Botloft goes to its agent. */
const FOLLOW_MS = 10 * 60 * 1000;

/**
 * What a finished run tells, given the failures in a row before it: only
 * the first failure of a row, so a routine that keeps failing does not
 * ring every time, and the run that works again after failing.
 */
export function noticeOf(status: RunStatus, streakBefore: number): "failed" | "recovered" | null {
  if (status === "failed") {
    return streakBefore === 0 ? "failed" : null;
  }
  return status === "done" && streakBefore > 0 ? "recovered" : null;
}

export function useRoutineAlerts(): void {
  const api = useApi();
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let last: { bot: BotId; at: number } | null = null;
    const stop = api.subscribe((event) => {
      if (event.name !== "routine.run" || !prefs.notifyRoutineFailed.get()) {
        return;
      }
      const state = store.getState();
      const routine = state.routines[event.params.routineId];
      const bot = routine ? state.bots[routine.botId] : undefined;
      if (!routine || !bot || bot.archivedAt !== null) {
        return;
      }
      // The routine's failures in a row before this run: its own update
      // comes after the run (spec 20.3).
      const kind = noticeOf(event.params.status, routine.failStreak);
      if (!kind) {
        return;
      }
      const sound = prefs.sound.get();
      if (host.window.inFront()) {
        if (sound) {
          playSound(kind === "failed" ? "needs" : "done");
        }
        return;
      }
      last = { bot: bot.id, at: Date.now() };
      const words = t().alerts;
      const title =
        kind === "failed" ? words.routineFailed(bot.name) : words.routineRecovered(bot.name);
      host.notify({ title, body: routine.name, sound }).catch(() => {});
    });
    let off: (() => void) | undefined;
    let alive = true;
    host.window
      .onReopened(() => {
        const bot = last && Date.now() - last.at < FOLLOW_MS ? last.bot : null;
        last = null;
        if (bot && store.getState().bots[bot]) {
          store.getState().selectBot(bot);
        }
      })
      .then(
        (unsubscribe) => {
          if (alive) {
            off = unsubscribe;
          } else {
            unsubscribe();
          }
        },
        () => {},
      );
    return () => {
      alive = false;
      stop();
      off?.();
    };
  }, [api, host, store]);
}
