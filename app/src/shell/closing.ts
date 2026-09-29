// What closing the window does (spec 15.1): with the icon near the clock,
// the window only hides; otherwise Botloft closes and the bots keep
// working in the background, or, if the owner chose so in Settings
// (`prefs.whenClosed`), stop until it opens again.

import { useEffect } from "react";
import type { Host } from "../lib/host";
import { prefs } from "./prefs";
import { trayShown } from "./tray";

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
        if (prefs.whenClosed.get() === "stop") {
          await host.window.hide().catch(() => {});
          await host.stopDaemon().catch(() => {});
          return "close";
        }
        if (trayShown()) {
          await host.window.hide().catch(() => {});
          return "stay";
        }
        return "close";
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
