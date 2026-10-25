// What the island needs from the native shell (spec 31.3): the daemon's
// port and the owner token, its own window's size and place, and the main
// window. The Tauri implementation is here; tests and the preview pass a
// fake.

import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { emitTo } from "@tauri-apps/api/event";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import type { DaemonStatus } from "../lib/host";
import { OPEN_BOT_EVENT } from "./event";

export interface IslandHost {
  /** Where the daemon listens and the owner token; null while it does not run. */
  daemon(): Promise<{ port: number; token: string } | null>;
  /** Gives the window this size, in CSS pixels, at the top middle of the main screen, and shows it without taking the focus. */
  show(width: number, height: number): Promise<void>;
  hide(): Promise<void>;
  /** Brings the main window forward on this agent's chat, or just forward. */
  openApp(botId?: string): Promise<void>;
}

export function tauriIslandHost(): IslandHost {
  const window = getCurrentWindow();
  return {
    async daemon() {
      const status = await invoke<DaemonStatus>("daemon_status");
      if (status.state !== "running") {
        return null;
      }
      return { port: status.port, token: await invoke<string>("read_owner_token") };
    },
    async show(width, height) {
      const screen = await primaryMonitor();
      await window.setSize(new LogicalSize(width, height));
      if (screen) {
        const scale = screen.scaleFactor;
        const left = screen.position.x / scale + (screen.size.width / scale - width) / 2;
        await window.setPosition(new LogicalPosition(Math.round(left), screen.position.y / scale));
      }
      if (!(await window.isVisible())) {
        await window.show();
      }
    },
    hide: () => window.hide(),
    openApp: (botId) => emitTo("main", OPEN_BOT_EVENT, botId ?? ""),
  };
}
