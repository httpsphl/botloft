// Dragging a crew to a new place in the sidebar. Pointer events, not the
// browser's drag and drop: the window's own file-drop handling takes over the
// latter on Windows.

import { type PointerEvent, type RefObject, useCallback, useRef, useState } from "react";
import type { CrewId } from "../../lib/protocol.gen";
import { moveBeside, setCrewOrder } from "./crewOrder";

/** How far the pointer moves before a press becomes a drag, in pixels. */
const THRESHOLD = 5;

export interface DropSpot {
  id: CrewId;
  after: boolean;
}

/** `ids` are the crews as listed; `list` holds one `<li data-crew-id>` for each. */
export function useCrewDrag(ids: readonly CrewId[], list: RefObject<HTMLElement | null>) {
  const [dragging, setDragging] = useState<CrewId | null>(null);
  const [over, setOver] = useState<DropSpot | null>(null);
  const swallow = useRef(false);
  const idsRef = useRef(ids);
  idsRef.current = ids;

  const spotAt = useCallback(
    (y: number): DropSpot | null => {
      const items = [
        ...(list.current?.querySelectorAll<HTMLElement>(":scope > [data-crew-id]") ?? []),
      ];
      for (const item of items) {
        const box = item.getBoundingClientRect();
        if (y <= box.bottom) {
          const id = item.dataset.crewId as CrewId;
          return { id, after: y > box.top + box.height / 2 && y > box.top + 16 };
        }
      }
      const last = items.at(-1);
      return last ? { id: last.dataset.crewId as CrewId, after: true } : null;
    },
    [list],
  );

  const start = useCallback(
    (id: CrewId, event: PointerEvent) => {
      if (event.button !== 0) {
        return;
      }
      const from = { x: event.clientX, y: event.clientY };
      let started = false;
      let spot: DropSpot | null = null;
      const finish = () => {
        window.removeEventListener("pointermove", move);
        window.removeEventListener("pointerup", up);
        window.removeEventListener("pointercancel", cancel);
        window.removeEventListener("keydown", key);
        setDragging(null);
        setOver(null);
      };
      const move = (e: globalThis.PointerEvent) => {
        if (!started && Math.hypot(e.clientX - from.x, e.clientY - from.y) < THRESHOLD) {
          return;
        }
        started = true;
        setDragging(id);
        spot = spotAt(e.clientY);
        setOver(spot);
      };
      const up = () => {
        if (started) {
          // The click that follows a drag must not open the crew.
          swallow.current = true;
          setTimeout(() => {
            swallow.current = false;
          }, 0);
          if (spot) {
            setCrewOrder(moveBeside(idsRef.current, id, spot.id, spot.after));
          }
        }
        finish();
      };
      const cancel = () => finish();
      const key = (e: KeyboardEvent) => {
        if (e.key === "Escape") {
          spot = null;
          finish();
        }
      };
      window.addEventListener("pointermove", move);
      window.addEventListener("pointerup", up);
      window.addEventListener("pointercancel", cancel);
      window.addEventListener("keydown", key);
    },
    [spotAt],
  );

  /** True once, right after a drag ended: the click to ignore. */
  const wasDrag = useCallback(() => swallow.current, []);
  return { dragging, over, start, wasDrag };
}
