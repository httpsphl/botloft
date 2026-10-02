import { beforeEach, expect, test, vi } from "vitest";
import type { TrayView } from "./host";

// The icons Windows shows, which outlive a page reload.
const shown: string[] = [];

vi.mock("@tauri-apps/api/app", () => ({ defaultWindowIcon: async () => null }));
vi.mock("@tauri-apps/api/image", () => ({ Image: {} }));
vi.mock("@tauri-apps/api/menu", () => {
  const item = { new: async () => ({ close: async () => {} }) };
  return { Menu: item, MenuItem: item, PredefinedMenuItem: item };
});
vi.mock("@tauri-apps/api/tray", () => ({
  TrayIcon: {
    new: async ({ id }: { id: string }) => {
      shown.push(id);
      return { setMenu: async () => {}, setTooltip: async () => {}, setIcon: async () => {} };
    },
    getById: async (id: string) => (shown.includes(id) ? {} : null),
    removeById: async (id: string) => {
      shown.splice(shown.indexOf(id), 1);
    },
  },
}));

const view: TrayView = {
  status: "No bot working",
  tooltip: "Botloft: No bot working",
  open: "Open",
  pause: "Pause",
  quit: "Quit",
  attention: false,
};
const actions = { open: () => {}, pause: () => {}, quit: () => {} };

beforeEach(() => {
  shown.length = 0;
  vi.resetModules();
});

test("two updates keep one icon", async () => {
  const { showTray } = await import("./tauriTray");
  await Promise.all([showTray(view, actions), showTray(view, actions)]);
  expect(shown).toEqual(["botloft"]);
});

test("a reloaded page replaces the icon it left behind", async () => {
  const before = await import("./tauriTray");
  await before.showTray(view, actions);
  vi.resetModules();
  const after = await import("./tauriTray");
  await after.showTray(view, actions);
  expect(shown).toEqual(["botloft"]);
});
