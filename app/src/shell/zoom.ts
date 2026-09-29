// How big the app is drawn (spec 15.3). The whole window scales, text,
// spacing and icons alike, like zooming a browser. 100% is the compact
// size the app was first drawn at; 125% is the default. Ctrl+= and Ctrl+-
// step through the levels and Ctrl+0 goes back to the default. The choice
// is a per-machine convenience kept in localStorage.

import { useEffect, useSyncExternalStore } from "react";
import type { Host } from "../lib/host";

export const ZOOM_LEVELS = [1, 1.1, 1.25, 1.5] as const;
export type ZoomLevel = (typeof ZOOM_LEVELS)[number];
export const DEFAULT_ZOOM: ZoomLevel = 1.25;

const KEY = "botloft.zoom";
const listeners = new Set<() => void>();

function stored(): ZoomLevel {
  try {
    const value = Number(localStorage.getItem(KEY));
    return ZOOM_LEVELS.find((level) => level === value) ?? DEFAULT_ZOOM;
  } catch {
    return DEFAULT_ZOOM;
  }
}

let zoom: ZoomLevel = stored();

export function currentZoom(): ZoomLevel {
  return zoom;
}

export function setZoom(next: ZoomLevel): void {
  zoom = next;
  try {
    localStorage.setItem(KEY, String(next));
  } catch {
    // Private storage off: the choice lasts for this window.
  }
  for (const listener of listeners) {
    listener();
  }
}

/** One level bigger (1) or smaller (-1), staying within the levels. */
export function stepZoom(direction: 1 | -1): void {
  const index = ZOOM_LEVELS.indexOf(zoom) + direction;
  const next = ZOOM_LEVELS[Math.min(ZOOM_LEVELS.length - 1, Math.max(0, index))];
  if (next !== undefined) {
    setZoom(next);
  }
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function useZoom(): ZoomLevel {
  return useSyncExternalStore(subscribe, () => zoom);
}

/** Draws the window at the chosen size and handles the zoom shortcuts. */
export function useZoomRoot(host: Host): void {
  const level = useZoom();
  useEffect(() => {
    host.setZoom(level).catch(() => {});
  }, [host, level]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!event.ctrlKey || event.altKey || event.metaKey) {
        return;
      }
      if (event.key === "=" || event.key === "+") {
        stepZoom(1);
      } else if (event.key === "-") {
        stepZoom(-1);
      } else if (event.key === "0") {
        setZoom(DEFAULT_ZOOM);
      } else {
        return;
      }
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
