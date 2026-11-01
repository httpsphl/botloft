// Telling the owner when a routine fails (spec 20.9, 15.1): a Windows
// notification with the agent and the routine, or the chime while Botloft
// is in front, and that agent's chat opened when the owner comes back from
// the notification.

import { useEffect } from "react";
import { t } from "../i18n";
import type { BotId } from "../lib/protocol.gen";
import { useApi, useAppStore, useHost } from "../store/context";
import { prefs } from "./prefs";
import { playSound } from "./sounds";

/** How long after a notification opening Botloft goes to its agent. */
const FOLLOW_MS = 10 * 60 * 1000;

export function useRoutineAlerts(): void {
  const api = useApi();
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let last: { bot: BotId; at: number } | null = null;
    const stop = api.subscribe((event) => {
      if (event.name !== "routine.run" || event.params.status !== "failed") {
        return;
      }
      if (!prefs.notifyRoutineFailed.get()) {
        return;
      }
      const state = store.getState();
      const routine = state.routines[event.params.routineId];
      const bot = routine ? state.bots[routine.botId] : undefined;
      if (!routine || !bot || bot.archivedAt !== null) {
        return;
      }
      const sound = prefs.sound.get();
      if (host.window.inFront()) {
        if (sound) {
          playSound("needs");
        }
        return;
      }
      last = { bot: bot.id, at: Date.now() };
      const words = t().alerts;
      host
        .notify({ title: words.routineFailed(bot.name), body: routine.name, sound })
        .catch(() => {});
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
