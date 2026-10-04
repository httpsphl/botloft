// Watching a bot's browser (spec 21.7): while the panel is open the app
// asks for its live frames and hears what the bot does, and stops asking
// when the panel closes. The browser's state itself lives in the store.

import { useCallback, useEffect, useState } from "react";
import type { Bot, BrowserAction, BrowserFrame } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";

export interface BrowserView {
  /** The newest picture of the page, if any came. */
  frame: BrowserFrame | null;
  /** What the bot did last, for the cursor and the caption. */
  action: BrowserAction | null;
  /** The daemon answered the watch: this connection may take the browser. */
  watched: boolean;
}

/**
 * The newest picture of each bot's page, so the panel opening again (from
 * the dock) shows it at once while the watch starts again.
 */
const lastFrames = new WeakMap<object, Map<string, BrowserFrame>>();

function framesOf(api: object): Map<string, BrowserFrame> {
  let frames = lastFrames.get(api);
  if (!frames) {
    frames = new Map();
    lastFrames.set(api, frames);
  }
  return frames;
}

export function useBrowserView(bot: Bot, watching: boolean): BrowserView {
  const api = useApi();
  const connected = useApp((state) => state.connection.kind === "open");
  const putBrowser = useApp((state) => state.putBrowser);
  const frames = framesOf(api);
  const [frame, setShown] = useState<BrowserFrame | null>(() => frames.get(bot.id) ?? null);
  const setFrame = useCallback(
    (next: BrowserFrame | null | ((current: BrowserFrame | null) => BrowserFrame | null)) =>
      setShown((current) => {
        const value = typeof next === "function" ? next(current) : next;
        if (value) {
          frames.set(value.botId, value);
        }
        return value;
      }),
    [frames],
  );
  const [action, setAction] = useState<BrowserAction | null>(null);
  const [watched, setWatched] = useState(false);

  // biome-ignore lint/correctness/useExhaustiveDependencies: another bot's page is not this bot's
  useEffect(() => {
    setFrame(frames.get(bot.id) ?? null);
    setAction(null);
  }, [bot.id]);

  useEffect(() => {
    if (!watching || !connected) {
      return;
    }
    let alive = true;
    const unsubscribe = api.subscribe((event) => {
      if (event.name === "browser.frame" && event.params.botId === bot.id) {
        setFrame(event.params);
      } else if (event.name === "browser.action" && event.params.botId === bot.id) {
        setAction(event.params);
      }
    });
    api.call("browser.watch", { botId: bot.id }).then(
      (view) => {
        if (alive) {
          setWatched(true);
          putBrowser(view.state);
          if (view.frame) {
            setFrame((current) => current ?? view.frame);
          }
        }
      },
      () => {},
    );
    return () => {
      alive = false;
      setWatched(false);
      unsubscribe();
      api.call("browser.unwatch").catch(() => {});
    };
  }, [api, bot.id, watching, connected, putBrowser, setFrame]);

  return { frame, action, watched };
}
