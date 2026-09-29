// The icon near the clock (spec 15.1) over Tauri's tray and menu. Updates
// run one after another, so two quick changes never make two icons.

import { defaultWindowIcon } from "@tauri-apps/api/app";
import { Image } from "@tauri-apps/api/image";
import { Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";
import { TrayIcon } from "@tauri-apps/api/tray";
import type { TrayActions, TrayView } from "./host";

let queue: Promise<unknown> = Promise.resolve();
let tray: TrayIcon | null = null;
let menu: Menu | null = null;
let actions: TrayActions | null = null;
const icons = new Map<boolean, Image>();

/** The app's icon, with an accent dot in the corner while something waits. */
async function icon(attention: boolean): Promise<Image | null> {
  const cached = icons.get(attention);
  if (cached) {
    return cached;
  }
  const plain = await defaultWindowIcon();
  if (!plain || !attention) {
    if (plain) {
      icons.set(false, plain);
    }
    return plain;
  }
  const { width, height } = await plain.size();
  const rgba = new Uint8Array(await plain.rgba());
  const radius = Math.max(3, Math.round(width * 0.24));
  const cx = width - radius - 0.5;
  const cy = height - radius - 0.5;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const distance = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
      if (distance <= radius) {
        const ring = distance > radius * 0.72;
        rgba.set(ring ? [11, 11, 11, 255] : [255, 122, 89, 255], (y * width + x) * 4);
      }
    }
  }
  const marked = await Image.new(rgba, width, height);
  icons.set(true, marked);
  return marked;
}

async function apply(view: TrayView): Promise<void> {
  const next = await Menu.new({
    items: [
      await MenuItem.new({ text: view.status, enabled: false }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({ text: view.open, action: () => actions?.open() }),
      await MenuItem.new({ text: view.pause, action: () => actions?.pause() }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({ text: view.quit, action: () => actions?.quit() }),
    ],
  });
  const image = await icon(view.attention);
  if (tray) {
    await tray.setMenu(next);
    await tray.setTooltip(view.tooltip);
    await tray.setIcon(image);
  } else {
    tray = await TrayIcon.new({
      id: "botloft",
      menu: next,
      tooltip: view.tooltip,
      showMenuOnLeftClick: false,
      ...(image ? { icon: image } : {}),
      action: (event) => {
        if (event.type === "Click" && event.button === "Left" && event.buttonState === "Up") {
          actions?.open();
        }
      },
    });
  }
  await menu?.close();
  menu = next;
}

export function showTray(view: TrayView, next: TrayActions): Promise<void> {
  actions = next;
  const run = queue.then(() => apply(view));
  queue = run.catch(() => {});
  return run;
}

export function hideTray(): Promise<void> {
  const run = queue.then(async () => {
    await tray?.close();
    await menu?.close();
    tray = null;
    menu = null;
  });
  queue = run.catch(() => {});
  return run;
}
