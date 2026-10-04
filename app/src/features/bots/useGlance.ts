// Bots look at each other when they talk (spec 15.3): when one bot sends
// another a message, the mascots of both glance toward each other for a
// moment, wherever the other one shows on screen (the sidebar, a crew's
// cards). A bot whose partner is nowhere on screen does not glance.

import { type RefObject, useEffect, useState } from "react";
import type { BotId } from "../../lib/protocol.gen";
import { useOptionalApi } from "../../store/context";

/** How long a glance lasts, as in mascot-glance.css. */
export const GLANCE_MS = 1600;

/** How far the eyes move, in the artwork's units (it is 634 across). */
const REACH = { x: 32, y: 22 };

export interface Glance {
  x: number;
  y: number;
}

/** Where `from` should look to see the nearest mascot of `botId`, if any shows. */
export function glanceToward(from: Element, botId: BotId): Glance | null {
  const here = from.getBoundingClientRect();
  const center = { x: here.left + here.width / 2, y: here.top + here.height / 2 };
  let nearest: { dx: number; dy: number; distance: number } | null = null;
  for (const other of document.querySelectorAll(`svg[data-bot="${botId}"]`)) {
    const there = other.getBoundingClientRect();
    if (there.width === 0 || there.height === 0) continue;
    const dx = there.left + there.width / 2 - center.x;
    const dy = there.top + there.height / 2 - center.y;
    const distance = Math.hypot(dx, dy);
    if (distance >= 1 && (!nearest || distance < nearest.distance)) {
      nearest = { dx, dy, distance };
    }
  }
  if (!nearest) return null;
  return {
    x: Math.round((nearest.dx / nearest.distance) * REACH.x),
    y: Math.round((nearest.dy / nearest.distance) * REACH.y),
  };
}

/** Where the mascot of `botId` (drawn in `svg`) is glancing now, if it is. */
export function useGlance(
  botId: BotId | undefined,
  svg: RefObject<SVGSVGElement | null>,
): Glance | null {
  const api = useOptionalApi();
  const [glance, setGlance] = useState<Glance | null>(null);
  useEffect(() => {
    if (!api || !botId) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const stop = api.subscribe((event) => {
      if (event.name !== "message.created") return;
      const { fromBotId, toBotId } = event.params;
      if (!fromBotId || fromBotId === toBotId) return;
      const partner = toBotId === botId ? fromBotId : fromBotId === botId ? toBotId : null;
      const toward = partner && svg.current ? glanceToward(svg.current, partner) : null;
      if (!toward) return;
      setGlance(toward);
      clearTimeout(timer);
      timer = setTimeout(() => setGlance(null), GLANCE_MS);
    });
    return () => {
      stop();
      clearTimeout(timer);
    };
  }, [api, botId, svg]);
  return glance;
}
