// `Host` over the Tauri commands in `src-tauri/src/lib.rs`.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Image } from "@tauri-apps/api/image";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { check } from "@tauri-apps/plugin-updater";
import type { AppUpdate, DaemonStatus, Host } from "./host";
import { hideTray, showTray } from "./tauriTray";

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

/**
 * Asks the release feed for a newer version (spec 15.5). A dev build never
 * offers to replace itself with the released app.
 */
async function checkForUpdate(): Promise<AppUpdate | null> {
  if (import.meta.env.DEV) {
    return null;
  }
  const update = await check();
  if (!update) {
    return null;
  }
  return {
    version: update.version,
    notes: update.body?.trim() || null,
    install: async (progress) => {
      let total = 0;
      let done = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            total = event.data.contentLength ?? 0;
            progress(total > 0 ? 0 : null);
            break;
          case "Progress":
            done += event.data.chunkLength;
            progress(total > 0 ? Math.min(1, done / total) : null);
            break;
          case "Finished":
            progress(1);
            break;
        }
      });
    },
  };
}

export function tauriHost(): Host {
  const window = getCurrentWindow();
  let overlay: Promise<Image> | undefined;
  return {
    daemonStatus: () => invoke<DaemonStatus>("daemon_status"),
    installDaemon: () => invoke<DaemonStatus>("daemon_install"),
    restartDaemon: () => invoke<DaemonStatus>("daemon_restart"),
    stopDaemon: () => invoke<void>("daemon_stop"),
    // A dev build never registers itself: it needs the dev server.
    setOpenAtSignIn: (on) =>
      import.meta.env.DEV ? Promise.resolve() : invoke<void>("open_at_sign_in", { on }),
    launchedAtSignIn: () => invoke<boolean>("launched_at_sign_in"),
    notify: async ({ title, body, sound }) => {
      if (!(await isPermissionGranted()) && (await requestPermission()) !== "granted") {
        return;
      }
      sendNotification({ title, body, ...(sound ? { sound: "Default" } : {}) });
    },
    // A dev build runs beside the installed app: its icon says which it is.
    showTray: (view, actions) =>
      showTray(import.meta.env.DEV ? { ...view, tooltip: `${view.tooltip} (dev)` } : view, actions),
    hideTray,
    signInToClaude: (path) => invoke<boolean>("claude_sign_in", { path }),
    setZoom: (factor) => getCurrentWebview().setZoom(factor),
    checkForUpdate,
    readOwnerToken: () => invoke<string>("read_owner_token"),
    openPath: (path) => invoke<void>("open_path", { path }),
    openFile: (path) => invoke<void>("open_file", { path }),
    revealFile: (path) => invoke<void>("reveal_file", { path }),
    pickFolder: async (title, start) => {
      const picked = await open({
        directory: true,
        multiple: false,
        title,
        ...(start ? { defaultPath: start } : {}),
      });
      return typeof picked === "string" ? picked : null;
    },
    openUrl: (url) => invoke<void>("open_url", { url }),
    window: {
      minimize: () => window.minimize(),
      toggleMaximize: () => window.toggleMaximize(),
      close: () => window.close(),
      hide: () => window.hide(),
      show: async () => {
        await window.show();
        await window.unminimize();
        await window.setFocus();
      },
      quit: () => window.destroy(),
      inFront: () => document.visibilityState === "visible" && document.hasFocus(),
      onCloseRequested: (before) =>
        window.onCloseRequested(async (event) => {
          if ((await before()) === "stay") {
            event.preventDefault();
          }
        }),
      onReopened: (listener) => listen("botloft://reopened", () => listener()),
      isMaximized: () => window.isMaximized(),
      onResized: (listener) => window.onResized(() => listener()),
      setAttention: async (on) => {
        overlay ??= Image.new(dot(), DOT, DOT);
        await window.setOverlayIcon(on ? await overlay : undefined);
      },
    },
  };
}
