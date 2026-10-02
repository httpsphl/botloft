import { beforeEach, expect, test, vi } from "vitest";

// Each `check` makes a new update; the first one's installer fails to start.
const made: { rid: number; closed: boolean }[] = [];
const rebuilt = vi.fn(async () => {});
const appWindow = { show: vi.fn(async () => {}), setFocus: vi.fn(async () => {}) };

vi.mock("@tauri-apps/api/core", () => ({ invoke: async () => {} }));
vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/api/image", () => ({ Image: {} }));
vi.mock("@tauri-apps/api/webview", () => ({ getCurrentWebview: () => ({}) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => appWindow }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: async () => null }));
vi.mock("@tauri-apps/plugin-notification", () => ({}));
vi.mock("./tauriTray", () => ({
  showTray: async () => {},
  hideTray: async () => {},
  rebuildTray: rebuilt,
}));
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: async () => {
    const handle = { rid: made.length + 1, closed: false };
    made.push(handle);
    return {
      version: "0.9.0",
      body: "",
      downloadAndInstall: async () => {
        if (handle.closed) {
          throw new Error(`The resource id ${handle.rid} is invalid.`);
        }
        if (handle.rid === 1) {
          handle.closed = true;
          throw new Error("The operation was canceled by the user. (os error 1223)");
        }
      },
      close: async () => {
        handle.closed = true;
      },
    };
  },
}));

beforeEach(() => {
  made.length = 0;
  vi.stubEnv("DEV", false);
});

test("a retry after an installer that did not start looks for the update again", async () => {
  const { tauriHost } = await import("./tauriHost");
  const update = await tauriHost().checkForUpdate();
  if (!update) {
    throw new Error("expected an update");
  }
  await expect(update.install(() => {})).rejects.toThrow("os error 1223");
  expect(rebuilt).toHaveBeenCalled();
  expect(appWindow.show).toHaveBeenCalled();
  await update.install(() => {});
  expect(made).toHaveLength(2);
});
