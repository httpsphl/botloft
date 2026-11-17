// The panel beside a bot's chat (spec 15.1): which one is open, kept per
// bot in the store so it comes back with the bot, and the one sliding
// closed while it leaves. A bot the owner has not touched the panels of
// opens with its overview (spec 33.2), when the window has room for it
// beside the chat.

import { useEffect, useMemo, useState } from "react";
import type { BotPanel } from "../../store/app";
import { useApp } from "../../store/context";

export type Side = BotPanel | null;

/** The panel beside a bot nobody chose one for yet. */
export const FIRST_PANEL: BotPanel = "overview";
/** Narrower windows (CSS px) keep the chat to itself until a panel is asked for. */
export const ROOM_FOR_FIRST_PANEL = 1100;

export function usePanel(botId: string) {
  // Looked at once each time the bot is opened.
  const [roomy] = useState(() => window.innerWidth >= ROOM_FOR_FIRST_PANEL);
  // Undefined: never chosen, so the overview. Null: the owner closed it.
  const side = useApp((state) => {
    const kept = state.panels[botId];
    return kept === undefined ? (roomy ? FIRST_PANEL : null) : kept;
  });
  const keepPanel = useApp((state) => state.setPanel);
  // The panel that came back is there at once; one opened after it slides.
  const [restored, setRestored] = useState(side !== null);
  const setSide = (panel: Side) => {
    setRestored(false);
    keepPanel(botId, panel);
  };
  // The panel on screen: the one open, or the last one while it slides
  // closed.
  const [leaving, setLeaving] = useState<Side>(null);
  useEffect(() => {
    if (side !== null) {
      setLeaving(side);
    }
  }, [side]);
  const closing = useMemo(
    () => (side === null && leaving !== null ? { closed: () => setLeaving(null) } : null),
    [side, leaving],
  );
  return { side, setSide, beside: side ?? leaving, restored, closing };
}
