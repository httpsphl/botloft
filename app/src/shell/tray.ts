// The icon near the clock (spec 15.1): while the bots keep working after
// the window closes and the owner wants it, Botloft stays there with what
// the bots are doing, a dot when something waits, and Open, Pause every
// crew and Quit.

import { useEffect } from "react";
import { t, useT } from "../i18n";
import { type AppState, crewList } from "../store/app";
import { useApi, useAppStore, useHost } from "../store/context";
import { attempt } from "../ui/toast";
import { attentionCount } from "./attention";
import { prefs, usePref } from "./prefs";

let shown = false;

/** Whether the icon is there: closing the window then only hides it. */
export function trayShown(): boolean {
  return shown;
}

function activeCrews(state: AppState) {
  return crewList(state).filter((crew) => crew.archivedAt === null);
}

function status(state: AppState): string {
  const words = t().alerts.tray;
  const bots = Object.values(state.bots).filter((bot) => bot.archivedAt === null);
  const needing = bots.find((bot) => bot.state === "needs_approval" || bot.state === "auth_error");
  if (needing) {
    return words.needsYou(needing.name);
  }
  if (attentionCount(state) > 0) {
    return words.waiting;
  }
  const working = bots.filter((bot) => bot.state === "busy").length;
  return working > 0 ? words.working(working) : words.idle;
}

export function useTray(): void {
  const host = useHost();
  const api = useApi();
  const store = useAppStore();
  const tray = usePref(prefs.tray);
  const keep = usePref(prefs.whenClosed) === "keep";
  const wanted = tray && keep;
  // The texts follow the language.
  const words = useT().alerts.tray;

  useEffect(() => {
    if (!wanted) {
      shown = false;
      host.hideTray().catch(() => {});
      return;
    }
    const pauseAll = () => {
      const crews = activeCrews(store.getState());
      const paused = !crews.every((crew) => crew.paused);
      const failed = t().crews.view.failed;
      attempt(paused ? failed.pause : failed.resume, async () => {
        for (const crew of crews.filter((each) => each.paused !== paused)) {
          store.getState().putCrew(await api.call("crews.setPaused", { crewId: crew.id, paused }));
        }
      });
    };
    let last = "";
    const draw = (state: AppState) => {
      const crews = activeCrews(state);
      const allPaused = crews.length > 0 && crews.every((crew) => crew.paused);
      const line = status(state);
      const view = {
        tooltip: words.tooltip(line),
        status: line,
        attention: attentionCount(state) > 0,
        open: words.open,
        pause: allPaused ? words.resume : words.pause,
        quit: words.quit,
      };
      const key = JSON.stringify(view);
      if (key === last) {
        return;
      }
      last = key;
      host
        .showTray(view, {
          open: () => {
            host.window.show().catch(() => {});
          },
          pause: pauseAll,
          quit: () => {
            host.window.quit().catch(() => {});
          },
        })
        .then(
          () => {
            shown = true;
          },
          () => {},
        );
    };
    draw(store.getState());
    const unsubscribe = store.subscribe(draw);
    return () => {
      unsubscribe();
      shown = false;
      host.hideTray().catch(() => {});
    };
  }, [host, api, store, wanted, words]);
}
