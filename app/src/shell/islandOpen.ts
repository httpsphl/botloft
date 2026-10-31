// "Open in Botloft" from the island (spec 31.3): the main window comes
// forward, on the agent's chat when the island named one it knows.

import { useEffect } from "react";
import { useAppStore, useHost } from "../store/context";

export function useIslandOpen(): void {
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let off: (() => void) | undefined;
    let alive = true;
    host.window
      .onOpenBot((botId) => {
        host.window.show().catch(() => {});
        if (botId && store.getState().bots[botId]) {
          store.getState().selectBot(botId);
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
      off?.();
    };
  }, [host, store]);
}
