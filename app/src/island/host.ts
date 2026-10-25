// What the island needs from the native shell (spec 31.3): the daemon's
// port and the owner token, its own window's size and place, whether the
// screen should stay quiet or the main window is in front, and the main
// window. The Tauri implementation is here; tests and the preview pass a
// fake.

import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { emitTo } from "@tauri-apps/api/event";
import {
  cursorPosition,
  getCurrentWindow,
  type Monitor,
  monitorFromPoint,
  primaryMonitor,
} from "@tauri-apps/api/window";
import type { DaemonStatus } from "../lib/host";
import { SYSTEM } from "../lib/system";
import { OPEN_BOT_EVENT } from "./event";

/** The screen the island sits on (spec 31.6). */
export type IslandScreen = "primary" | "mouse";

export interface IslandHost {
  /** Whether the island runs on this system: only Windows for now (spec 31.10). */
  readonly supported: boolean;
  /** Where the daemon listens and the owner token; null while it does not run. */
  daemon(): Promise<{ port: number; token: string } | null>;
  /**
   * Gives the window this size, in CSS pixels, at the top middle of the
   * screen, and shows it without taking the focus.
   */
  show(width: number, height: number, screen: IslandScreen): Promise<void>;
  hide(): Promise<void>;
  /** Brings the main window forward on this agent's chat, or just forward. */
  openApp(botId?: string): Promise<void>;
  /** Windows would hold back a notification now: full screen, a presentation, "Do not disturb". */
  quiet(): Promise<boolean>;
  /** The main window is on screen and has the focus. */
  appInFront(): Promise<boolean>;
}

async function monitorOf(screen: IslandScreen): Promise<Monitor | null> {
  if (screen === "mouse") {
    const at = await cursorPosition();
    const under = await monitorFromPoint(at.x, at.y);
    if (under) {
      return under;
    }
  }
  return primaryMonitor();
}

export function tauriIslandHost(): IslandHost {
  const window = getCurrentWindow();
  return {
    supported: SYSTEM === "Windows",
    async daemon() {
      const status = await invoke<DaemonStatus>("daemon_status");
      if (status.state !== "running") {
        return null;
      }
      return { port: status.port, token: await invoke<string>("read_owner_token") };
    },
    async show(width, height, screen) {
      const monitor = await monitorOf(screen);
      await window.setSize(new LogicalSize(width, height));
      if (monitor) {
        const scale = monitor.scaleFactor;
        const left = monitor.position.x / scale + (monitor.size.width / scale - width) / 2;
        await window.setPosition(new LogicalPosition(Math.round(left), monitor.position.y / scale));
      }
      if (!(await window.isVisible())) {
        await window.show();
      }
    },
    hide: () => window.hide(),
    openApp: (botId) => emitTo("main", OPEN_BOT_EVENT, botId ?? ""),
    quiet: () => invoke<boolean>("quiet_now"),
    appInFront: () => invoke<boolean>("main_in_front"),
  };
}
