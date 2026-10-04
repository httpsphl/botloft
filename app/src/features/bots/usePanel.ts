// The panel beside a bot's chat (spec 15.1): which one is open, kept per
// bot in the store so it comes back with the bot, and the one sliding
// closed while it leaves.

import { useEffect, useMemo, useState } from "react";
import type { BotPanel } from "../../store/app";
import { useApp } from "../../store/context";

export type Side = BotPanel | null;

export function usePanel(botId: string) {
  const side = useApp((state) => state.panels[botId] ?? null);
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
