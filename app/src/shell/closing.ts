// What closing the window does (spec 15.1): Botloft keeps the bots working
// in the background, or, if the owner chose so in Settings
// (`prefs.whenClosed`), stops them until it opens again.

import { useEffect } from "react";
import type { Host } from "../lib/host";
import { prefs } from "./prefs";

/**
 * Stops the bots as the window closes, when the owner chose so. The window
 * hides first, since stopping takes a moment; if it fails, it closes all
 * the same.
 */
export function useCloseBehavior(host: Host): void {
  useEffect(() => {
    let off: (() => void) | undefined;
    let alive = true;
    host.window
      .onCloseRequested(async () => {
        if (prefs.whenClosed.get() !== "stop") {
          return;
        }
        await host.window.hide().catch(() => {});
        await host.stopDaemon().catch(() => {});
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
  }, [host]);
}
