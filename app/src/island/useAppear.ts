// When the island opens on its own (spec 31.6): it comes down on Home when
// a new request or question arrives, unless the main window is in front or
// Windows keeps the screen quiet; and it peeks while the mouse rests on the
// pill, closing a second after it leaves unless the owner clicked inside.

import { useCallback, useEffect, useRef, useState } from "react";
import type { IslandHost } from "./host";

/** How long the mouse rests on the pill before the island peeks. */
export const PEEK_MS = 300;
/** How long after the mouse leaves a peek closes. */
export const LEAVE_MS = 1000;

export interface Appear {
  open: boolean;
  setOpen: (open: boolean) => void;
  /** Mouse handlers for the island's outline. */
  hover: {
    onMouseEnter: () => void;
    onMouseLeave: () => void;
    onPointerDown: () => void;
  };
}

export function useAppear({
  host,
  enabled,
  loaded,
  waitingIds,
  onArrive,
}: {
  host: IslandHost;
  enabled: boolean;
  /** The first read is in: what waits then is not news. */
  loaded: boolean;
  waitingIds: string[];
  /** A new request came down: show it. */
  onArrive: () => void;
}): Appear {
  const [open, setOpenState] = useState(false);
  const peeking = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const seen = useRef<Set<string> | null>(null);
  const arrive = useRef(onArrive);
  arrive.current = onArrive;

  const setOpen = useCallback((next: boolean) => {
    clearTimeout(timer.current);
    peeking.current = false;
    setOpenState(next);
  }, []);

  const key = waitingIds.join(",");
  useEffect(() => {
    const ids = key === "" ? [] : key.split(",");
    if (seen.current === null) {
      if (loaded) {
        seen.current = new Set(ids);
      }
      return;
    }
    const known = seen.current;
    const fresh = ids.filter((id) => !known.has(id));
    for (const id of fresh) {
      known.add(id);
    }
    if (fresh.length === 0 || !enabled) {
      return;
    }
    let alive = true;
    Promise.all([host.quiet(), host.appInFront()]).then(
      ([quiet, front]) => {
        if (alive && !quiet && !front) {
          arrive.current();
          setOpen(true);
        }
      },
      () => {},
    );
    return () => {
      alive = false;
    };
  }, [host, key, loaded, enabled, setOpen]);

  useEffect(() => () => clearTimeout(timer.current), []);

  const hover = {
    onMouseEnter: () => {
      clearTimeout(timer.current);
      if (open) {
        return;
      }
      timer.current = setTimeout(() => {
        peeking.current = true;
        setOpenState(true);
      }, PEEK_MS);
    },
    onMouseLeave: () => {
      clearTimeout(timer.current);
      if (peeking.current) {
        timer.current = setTimeout(() => {
          peeking.current = false;
          setOpenState(false);
        }, LEAVE_MS);
      }
    },
    // A click inside keeps a peek open.
    onPointerDown: () => {
      peeking.current = false;
    },
  };

  return { open, setOpen, hover };
}
