// What a bot does on the owner's desktop, live (spec 24.9): its state and
// the picture of the window it is using, while its panel is open.

import { useEffect, useState } from "react";
import type { BotId, DesktopFrame, DesktopState } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export interface DesktopView {
  /** `null` until the daemon answers. */
  state: DesktopState | null;
  frame: DesktopFrame | null;
}

export function useDesktopView(botId: BotId): DesktopView {
  const api = useApi();
  const [view, setView] = useState<DesktopView>({ state: null, frame: null });
  useEffect(() => {
    let alive = true;
    setView({ state: null, frame: null });
    const stop = api.subscribe((event) => {
      if (event.name === "desktop.changed" && event.params.botId === botId) {
        setView((now) => ({ ...now, state: event.params }));
      }
      if (event.name === "desktop.frame" && event.params.botId === botId) {
        setView((now) => ({ ...now, frame: event.params }));
      }
    });
    api.call("desktop.watch", { botId }).then(
      // What arrived meanwhile is newer than the answer.
      (watched) =>
        alive &&
        setView((now) => ({
          state: now.state ?? watched.state,
          frame: now.frame ?? watched.frame,
        })),
      // An older daemon has no desktop: nothing to show.
      () =>
        alive &&
        setView({
          state: { botId, window: null, action: null, at: null, stopped: false },
          frame: null,
        }),
    );
    return () => {
      alive = false;
      stop();
      api.call("desktop.unwatch").catch(() => {});
    };
  }, [api, botId]);
  return view;
}
