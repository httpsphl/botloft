// How wide the crews list on the left is: the owner drags its right edge
// (features/crews/SidebarResizer.tsx). The width lives in a CSS variable on
// the root, so dragging only changes a style and the list, the slot that
// holds it and the chat all follow. It is remembered like the width of the
// panels on the right (ui/SidePanel.tsx).

import { useSyncExternalStore } from "react";

export const SIDEBAR_DEFAULT_PX = 288;
export const SIDEBAR_MIN_PX = 224;
export const SIDEBAR_MAX_PX = 560;

const KEY = "botloft.panel.sidebar";
const listeners = new Set<() => void>();

const clamp = (width: number) =>
  Math.round(Math.min(SIDEBAR_MAX_PX, Math.max(SIDEBAR_MIN_PX, width)));

function read(): number {
  try {
    const value = Number(localStorage.getItem(KEY));
    return Number.isFinite(value) && value > 0 ? clamp(value) : SIDEBAR_DEFAULT_PX;
  } catch {
    return SIDEBAR_DEFAULT_PX;
  }
}

let width = read();

function apply(): void {
  if (typeof document !== "undefined") {
    document.documentElement.style.setProperty("--sidebar-width", `${width}px`);
  }
}
apply();

export const sidebarWidth = {
  get: () => width,
  /** `keep` also remembers it; while dragging only the screen follows. */
  set(next: number, keep = false): void {
    width = clamp(next);
    apply();
    if (keep) {
      try {
        localStorage.setItem(KEY, String(width));
      } catch {
        // A convenience: the width just is not kept.
      }
    }
    for (const listener of listeners) {
      listener();
    }
  },
  reset(): void {
    this.set(SIDEBAR_DEFAULT_PX, true);
  },
  subscribe(listener: () => void): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },
};

export function useSidebarWidth(): number {
  return useSyncExternalStore(sidebarWidth.subscribe, sidebarWidth.get, sidebarWidth.get);
}
