// The panel beside the chat follows what the bot starts doing (spec 15.1),
// so the owner sees it happen: its browser starting or a request for a
// hand opens the browser (spec 21.8), and a screen it starts writing opens
// the design area (spec 22.5), and the bot starting to use the owner's
// desktop after a pause (a desktop tool starting in the chat, or the daemon
// saying it used a window) opens the desktop panel (spec 24.9), whatever
// panel was open. Each opens once per
// start; closing it leaves the button's dot. When the browser goes to rest
// (spec 21.2) its panel closes, and it opens again once the bot wakes it.
// The browser in the owner's hands is never taken away. The owner can turn this off in Settings
// (`prefs.followBot`): then only the buttons' dots say so. Coming back to a
// bot is opening it again: the panel the owner left is the one open (the
// store keeps it), and what opens by itself here takes its place.

import { useEffect, useRef } from "react";
import type { Bot } from "../../lib/protocol.gen";
import { prefs } from "../../shell/prefs";
import { useApi, useApp } from "../../store/context";
import { isDesktopTool } from "../desktop/showDesktop";

export type Followed = "browser" | "screens" | "desktop";

/** A use of the desktop this long after the last one starts a new run. */
const DESKTOP_PAUSE = 2 * 60_000;

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
  const api = useApi();
  // The latest of what the desktop listener needs, without listening anew.
  const now = useRef({ side, held, open });
  now.current = { side, held, open };

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

  useEffect(() => {
    let last: number | null = null;
    // A use of the desktop: a desktop tool starting in the chat, or the
    // daemon saying the bot read or acted in a window.
    const used = (at: number) => {
      if (at === last) {
        return;
      }
      const was = last;
      last = at;
      if (was !== null && Math.abs(at - was) < DESKTOP_PAUSE) {
        return;
      }
      const { side, held, open } = now.current;
      if (side !== "desktop" && !held && prefs.followBot.get()) {
        open("desktop");
      }
    };
    return api.subscribe((event) => {
      if (event.name === "chat.item" && event.params.item.botId === bot.id) {
        const { body, createdAt } = event.params.item;
        if (body.kind === "tool" && body.status === "running" && isDesktopTool(body.name)) {
          used(createdAt);
        }
      } else if (event.name === "desktop.changed" && event.params.botId === bot.id) {
        const { at, window, stopped } = event.params;
        if (at !== null && window && !stopped) {
          used(at);
        }
      }
    });
  }, [api, bot.id]);
}
