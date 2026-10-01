// The panel beside the chat follows what the bot starts doing (spec 15.1),
// so the owner sees it happen: its browser starting or a request for a
// hand opens the browser (spec 21.8), and a screen it starts writing opens
// the design area (spec 22.5), whatever panel was open. Each opens once per
// start; closing it leaves the button's dot. When the browser goes to rest
// (spec 21.2) its panel closes, and it opens again once the bot wakes it.
// The browser in the owner's hands is never taken away. The owner can turn this off in Settings
// (`prefs.followBot`): then only the buttons' dots say so. Coming back to a
// bot is opening it again: the panel the owner left is the one open (the
// store keeps it), and what opens by itself here takes its place.

import { useEffect, useRef } from "react";
import type { Bot } from "../../lib/protocol.gen";
import { prefs } from "../../shell/prefs";
import { useApp } from "../../store/context";

export type Followed = "browser" | "screens";

export function useFollowBot({
  bot,
  side,
  writing,
  open,
  close,
}: {
  bot: Pick<Bot, "id">;
  /** The panel open now, if any. */
  side: string | null;
  /** The screen the bot is writing now, if any. */
  writing: string | null;
  open(panel: Followed): void;
  close(panel: Followed): void;
}): void {
  const browser = useApp((state) => state.browsers[bot.id]);
  // A browser at rest is open, but the bot is not using it.
  const resting = browser?.status === "open" && browser.resting;
  const browsing = browser?.status === "starting" || (browser?.status === "open" && !resting);
  const asking = Boolean(browser?.ask);
  const held = browser?.control === "owner";
  // What was going on when the bot was opened: a browser left open stays
  // behind its button, a request for a hand does not.
  const before = useRef<{ browsing: boolean; asking: boolean } | null>(null);
  const drawn = useRef(new Set<string>());

  // biome-ignore lint/correctness/useExhaustiveDependencies: another bot starts afresh
  useEffect(() => {
    before.current = null;
    drawn.current = new Set();
  }, [bot.id]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: only a change in what the bot does opens it
  useEffect(() => {
    const was = before.current ?? { browsing, asking: false };
    before.current = { browsing, asking };
    if ((asking && !was.asking) || (browsing && !was.browsing)) {
      if (side !== "browser" && prefs.followBot.get()) {
        open("browser");
      }
    } else if (resting && was.browsing && !asking && !held) {
      if (side === "browser" && prefs.followBot.get()) {
        close("browser");
      }
    }
  }, [bot.id, browsing, asking]);

  // biome-ignore lint/correctness/useExhaustiveDependencies: only a new screen being written opens it
  useEffect(() => {
    const path = writing?.toLowerCase();
    if (!path || drawn.current.has(path)) {
      return;
    }
    drawn.current.add(path);
    if (side === "screens" || (held && side === "browser") || !prefs.followBot.get()) {
      return;
    }
    open("screens");
  }, [bot.id, writing]);
}
