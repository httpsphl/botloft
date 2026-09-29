// Watching a bot's browser (spec 21.7): while the panel is open the app
// asks for its live frames and hears what the bot does, and stops asking
// when the panel closes. The browser's state itself lives in the store.

import { useEffect, useState } from "react";
import type { Bot, BrowserAction, BrowserFrame } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";

export interface BrowserView {
  /** The newest picture of the page, if any came. */
  frame: BrowserFrame | null;
  /** What the bot did last, for the cursor and the caption. */
  action: BrowserAction | null;
}

export function useBrowserView(bot: Bot, watching: boolean): BrowserView {
  const api = useApi();
  const connected = useApp((state) => state.connection.kind === "open");
  const putBrowser = useApp((state) => state.putBrowser);
  const [frame, setFrame] = useState<BrowserFrame | null>(null);
  const [action, setAction] = useState<BrowserAction | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: another bot's page is not this bot's
  useEffect(() => {
    setFrame(null);
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
      unsubscribe();
      api.call("browser.unwatch").catch(() => {});
    };
  }, [api, bot.id, watching, connected, putBrowser]);

  return { frame, action };
}
