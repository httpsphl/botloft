// The island coming and going (spec 31.2): it comes out of the top edge as
// a sliver and grows, and when it leaves the content fades and the shape
// goes back into the edge before the window hides. The window grows before
// the shape and shrinks only after it, so no frame of it is cut.

import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { IslandHost, IslandScreen } from "./host";

/** How long the shape takes to grow or shrink (island.css). */
export const MORPH_MS = 400;
/** How long it takes to go back into the edge (island.css). */
export const GONE_MS = 380;
/** How long the sliver waits for frames before it grows anyway. */
const ENTER_FALLBACK_MS = 60;
/** What it comes out of and goes back into: a sliver at the top edge. */
const SEED = { width: 120, height: 0 };

export type Phase = "enter" | "shown" | "leave";

interface Size {
  width: number;
  height: number;
}

/** "Less motion" in Settings, or Windows asking for less (spec 15.3). */
function still(): boolean {
  if (typeof document !== "undefined" && document.documentElement.dataset.motion === "less") {
    return true;
  }
  return typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function usePresence({
  host,
  visible,
  expanded,
  width,
  height,
  screen,
}: {
  host: IslandHost;
  visible: boolean;
  expanded: boolean;
  width: number;
  height: number;
  screen: IslandScreen;
}): {
  /** Drawn at all: also while it leaves. */
  present: boolean;
  phase: Phase;
  /** Whether the panel's content shows: while it leaves, what showed before. */
  panel: boolean;
  /** The shape's size now; the window may be larger while it changes. */
  shape: Size;
} {
  const [present, setPresent] = useState(false);
  const [phase, setPhase] = useState<Phase>("shown");
  const last = useRef({ panel: expanded, size: { width, height } });
  const window = useRef<Size | null>(null);
  const leaving = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const first = useRef(true);

  // What showed last, for the way out.
  useLayoutEffect(() => {
    if (visible) {
      last.current = { panel: expanded, size: { width, height } };
    }
  }, [visible, expanded, width, height]);

  useLayoutEffect(() => {
    // A window left showing by a page that reloaded hides at once.
    if (first.current) {
      first.current = false;
      if (!visible) {
        host.hide().catch(() => {});
        return;
      }
    }
    if (visible) {
      clearTimeout(leaving.current);
      if (!present) {
        setPresent(true);
        setPhase(still() ? "shown" : "enter");
      } else if (phase === "leave") {
        setPhase("shown");
      }
      return;
    }
    if (!present || phase === "leave") {
      return;
    }
    const gone = () => {
      setPresent(false);
      setPhase("shown");
      window.current = null;
      host.hide().catch(() => {});
    };
    if (still()) {
      gone();
      return;
    }
    setPhase("leave");
    leaving.current = setTimeout(gone, GONE_MS);
  }, [host, visible, present, phase]);

  useEffect(() => () => clearTimeout(leaving.current), []);

  // The sliver is drawn for a frame, then it grows from there. Frames stop
  // in a window the system takes for hidden, so a timer stands in.
  useEffect(() => {
    if (phase !== "enter") {
      return;
    }
    const grow = () => setPhase((now) => (now === "enter" ? "shown" : now));
    let second = 0;
    const first = requestAnimationFrame(() => {
      second = requestAnimationFrame(grow);
    });
    const fallback = setTimeout(grow, ENTER_FALLBACK_MS);
    return () => {
      cancelAnimationFrame(first);
      cancelAnimationFrame(second);
      clearTimeout(fallback);
    };
  }, [phase]);

  // The window: as large as the shape will be now, and only smaller once
  // the shape has shrunk.
  useLayoutEffect(() => {
    if (!present) {
      return;
    }
    const target = phase === "leave" ? last.current.size : { width, height };
    const now = window.current;
    const room = now
      ? { width: Math.max(now.width, target.width), height: Math.max(now.height, target.height) }
      : target;
    window.current = room;
    host.show(room.width, room.height, screen).catch(() => {});
    if (room.width === target.width && room.height === target.height) {
      return;
    }
    const timer = setTimeout(
      () => {
        window.current = target;
        host.show(target.width, target.height, screen).catch(() => {});
      },
      still() ? 0 : MORPH_MS,
    );
    return () => clearTimeout(timer);
  }, [host, present, phase, width, height, screen]);

  return {
    present,
    phase,
    panel: phase === "leave" ? last.current.panel : expanded,
    shape: phase === "shown" ? { width, height } : SEED,
  };
}
