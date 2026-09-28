// `Host` over the Tauri commands in `src-tauri/src/lib.rs`.

import { invoke } from "@tauri-apps/api/core";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { DaemonStatus, Host } from "./host";

const DOT = 16;

/** A 16 px accent dot with a dark ring, drawn as RGBA for the overlay. */
function dot(): Uint8Array {
  const rgba = new Uint8Array(DOT * DOT * 4);
  const center = DOT / 2;
  for (let y = 0; y < DOT; y += 1) {
    for (let x = 0; x < DOT; x += 1) {
      const distance = Math.hypot(x + 0.5 - center, y + 0.5 - center);
      const coverage = Math.min(1, Math.max(0, 7.5 - distance));
      const ring = distance > 5.5;
      const at = (y * DOT + x) * 4;
      rgba.set(ring ? [11, 11, 11] : [255, 122, 89], at);
      rgba[at + 3] = Math.round(coverage * 255);
    }
  }
  return rgba;
}

export function tauriHost(): Host {
  const window = getCurrentWindow();
  let overlay: Promise<Image> | undefined;
  return {
    daemonStatus: () => invoke<DaemonStatus>("daemon_status"),
    startDaemon: () => invoke<DaemonStatus>("daemon_start"),
    readOwnerToken: () => invoke<string>("read_owner_token"),
    openPath: (path) => invoke<void>("open_path", { path }),
    window: {
      minimize: () => window.minimize(),
      toggleMaximize: () => window.toggleMaximize(),
      close: () => window.close(),
      isMaximized: () => window.isMaximized(),
      onResized: (listener) => window.onResized(() => listener()),
      setAttention: async (on) => {
        overlay ??= Image.new(dot(), DOT, DOT);
        await window.setOverlayIcon(on ? await overlay : undefined);
      },
    },
  };
}
