// The room the panel has for the bot's page (spec 21.8): measured as the
// panel changes and told to the daemon, so the page takes its shape (spec
// 21.3).

import { useEffect, useLayoutEffect, useState } from "react";
import type { Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export interface Size {
  width: number;
  height: number;
}

/** How long the room stays the same before the daemon hears about it. */
const SETTLED_MS = 300;

/**
 * The room inside an element, following it as it changes: `null` until it
 * has any. The second value goes in the element's `ref`.
 */
export function useRoom(): [Size | null, (node: HTMLElement | null) => void] {
  const [node, setNode] = useState<HTMLElement | null>(null);
  const [room, setRoom] = useState<Size | null>(null);
  useLayoutEffect(() => {
    if (!node) {
      setRoom(null);
      return;
    }
    const measure = () => {
      const width = node.clientWidth;
      const height = node.clientHeight;
      setRoom((was) => {
        if (width <= 0 || height <= 0) {
          return null;
        }
        return was?.width === width && was.height === height ? was : { width, height };
      });
    };
    measure();
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(node);
    return () => observer.disconnect();
  }, [node]);
  return [room, setNode];
}

/**
 * Tells the daemon the room the page has, once it stops changing, while
 * this connection watches the bot's browser, and how sharp the screen is,
 * so the page is drawn with as many pixels as show it (spec 21.3).
 */
export function useFitPage(bot: Pick<Bot, "id">, room: Size | null, watched: boolean): void {
  const api = useApi();
  const known = room !== null;
  const width = Math.max(1, room?.width ?? 0);
  const height = Math.max(1, room?.height ?? 0);
  // A change of the app's zoom changes the room too, which reads it again.
  const scale = Math.round((typeof devicePixelRatio === "number" ? devicePixelRatio : 1) * 100);
  useEffect(() => {
    if (!watched || !known) {
      return;
    }
    const timer = setTimeout(() => {
      // The next change of the room asks again.
      api.call("browser.resize", { botId: bot.id, width, height, scale }).catch(() => {});
    }, SETTLED_MS);
    return () => clearTimeout(timer);
  }, [api, bot.id, watched, known, width, height, scale]);
}
