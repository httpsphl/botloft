// Telling the owner when a bot needs them or finishes (spec 15.1): a
// Windows notification while Botloft is not in front (with the Windows
// sound, if the owner wants sound), the chime while it is, and that bot
// opened when the owner comes back from the notification.

import { useEffect } from "react";
import { toolTitle } from "../features/chat/toolNames";
import { t } from "../i18n";
import type { Bot, BotId, BotState } from "../lib/protocol.gen";
import type { AppState } from "../store/app";
import { useAppStore, useHost } from "../store/context";
import { chime } from "./chime";
import { prefs } from "./prefs";

/** How long after a notification opening Botloft goes to its bot. */
const FOLLOW_MS = 10 * 60 * 1000;

type Kind = "approval" | "signIn" | "done";

function kindOf(before: BotState | undefined, now: BotState): Kind | null {
  if (before === undefined || before === now) {
    return null;
  }
  if (now === "needs_approval") {
    return "approval";
  }
  if (now === "auth_error") {
    return "signIn";
  }
  return before === "busy" && now === "idle" ? "done" : null;
}

function notice(state: AppState, bot: Bot, kind: Kind): { title: string; body: string } {
  const words = t();
  const crew = state.crews[bot.crewId]?.name ?? "";
  switch (kind) {
    case "approval": {
      const tool = bot.lastActivity?.kind === "approval" ? bot.lastActivity.tool : null;
      return {
        title: words.alerts.approval(bot.name),
        body: tool ? toolTitle(tool, words.tools) : words.alerts.crew(crew),
      };
    }
    case "signIn":
      return { title: words.alerts.signIn(bot.name), body: words.alerts.signInBody };
    case "done":
      return { title: words.alerts.done(bot.name), body: words.alerts.crew(crew) };
  }
}

export function useBotAlerts(): void {
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let seen = new Map<BotId, BotState>();
    let last: { bot: BotId; at: number } | null = null;

    const follow = (state: AppState) => {
      const now = new Map(Object.values(state.bots).map((bot) => [bot.id, bot.state]));
      // What was there on (re)connecting is not news.
      const loading = !state.loaded;
      const before = seen;
      seen = now;
      if (loading || before.size === 0) {
        return;
      }
      for (const bot of Object.values(state.bots)) {
        const kind = bot.archivedAt === null ? kindOf(before.get(bot.id), bot.state) : null;
        const wanted = kind === "done" ? prefs.notifyDone.get() : prefs.notifyNeeds.get();
        if (!kind || !wanted) {
          continue;
        }
        const sound = prefs.sound.get();
        if (host.window.inFront()) {
          if (sound) {
            chime(kind === "done" ? "done" : "needs");
          }
          continue;
        }
        last = { bot: bot.id, at: Date.now() };
        host.notify({ ...notice(state, bot, kind), sound }).catch(() => {});
      }
    };

    seen = new Map(Object.values(store.getState().bots).map((bot) => [bot.id, bot.state]));
    const unsubscribe = store.subscribe(follow);
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
        (unsubscribeReopen) => {
          if (alive) {
            off = unsubscribeReopen;
          } else {
            unsubscribeReopen();
          }
        },
        () => {},
      );
    return () => {
      alive = false;
      unsubscribe();
      off?.();
    };
  }, [host, store]);
}
