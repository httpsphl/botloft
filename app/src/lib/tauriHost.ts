// `Host` over the Tauri commands in `src-tauri/src/lib.rs`.

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { DaemonStatus, Host } from "./host";

export function tauriHost(): Host {
  const window = getCurrentWindow();
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
    },
  };
}
