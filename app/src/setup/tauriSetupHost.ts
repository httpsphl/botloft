// `SetupHost` over the commands in `src-setup/src/main.rs`.

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { SetupHost, SetupState } from "./host";

export function tauriSetupHost(): SetupHost {
  const window = getCurrentWindow();
  return {
    state: () => invoke<SetupState>("setup_state"),
    install: () => invoke("setup_install"),
    openApp: () => invoke("setup_open_app"),
    classic: () => invoke("setup_classic"),
    close: () => {
      window.close().catch(() => {});
    },
    show: () => {
      window.show().catch(() => {});
    },
  };
}
