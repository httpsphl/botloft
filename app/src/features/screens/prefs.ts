// What the owner chose in the design area, kept in localStorage (spec
// 22.5): the board's zoom and the device of each screen. Conveniences:
// without storage the defaults just come back.

import { type RefObject, useEffect, useState } from "react";
import type { Screen, ScreenDevice } from "../../lib/protocol.gen";
import { DEVICES } from "./LiveFrame";

const ZOOM_KEY = "botloft.screens.zoom";
const DEVICES_KEY = "botloft.screens.devices";
export const ZOOMS = [0.15, 0.25, 0.35, 0.5, 0.75, 1] as const;
/** The board's scale, or "fit": the widest screen fills the width, up to half size. */
export type Zoom = number | "fit";

function read<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

function write(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Not kept; the default comes back next time.
  }
}

export function useZoom(): [Zoom, (zoom: Zoom) => void] {
  const [zoom, setZoom] = useState<Zoom>(() => {
    const saved = read<unknown>(ZOOM_KEY, "fit");
    return typeof saved === "number" && saved > 0 && saved <= 1 ? saved : "fit";
  });
  return [
    zoom,
    (next) => {
      setZoom(next);
      write(ZOOM_KEY, next);
    },
  ];
}

/** The device each screen shows on: the owner's choice, the file's hint, or a computer. */
export function useDevices(): [
  (screen: Screen) => ScreenDevice,
  (path: string, device: ScreenDevice) => void,
] {
  const [chosen, setChosen] = useState<Record<string, ScreenDevice>>(() =>
    read<Record<string, ScreenDevice>>(DEVICES_KEY, {}),
  );
  const deviceOf = (screen: Screen): ScreenDevice => {
    const picked = chosen[screen.path.toLowerCase()];
    return picked && picked in DEVICES ? picked : (screen.device ?? "desktop");
  };
  const choose = (path: string, device: ScreenDevice) => {
    const next = { ...chosen, [path.toLowerCase()]: device };
    setChosen(next);
    write(DEVICES_KEY, next);
  };
  return [deviceOf, choose];
}

/** The scale of `zoom` for screens as wide as `widest`, in a board `width` wide. */
export function scaleOf(zoom: Zoom, width: number, widest: number): number {
  if (zoom !== "fit") {
    return zoom;
  }
  return Math.max(0.1, Math.min(0.5, (width - 48) / widest));
}

/** The width of an element, following it as it changes. */
export function useWidth(ref: RefObject<HTMLElement | null>, fallback = 600): number {
  const [width, setWidth] = useState(fallback);
  useEffect(() => {
    const element = ref.current;
    if (!element) {
      return;
    }
    const measure = () => setWidth(element.clientWidth || fallback);
    measure();
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [ref, fallback]);
  return width;
}
