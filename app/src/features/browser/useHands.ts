// The owner's hands in a bot's browser (spec 21.10): taking it, giving it
// back, sending what they do on the page, and moving between its tabs.
// Events go out in order and without waiting for each other: the daemon
// queues them for the page.

import { useCallback, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserInput, BrowserState } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { attempt } from "../../ui/toast";

export interface Hands {
  /** The owner has the browser. */
  held: boolean;
  busy: boolean;
  take(): Promise<void>;
  release(): Promise<void>;
  send(input: BrowserInput): void;
  /** Opens a blank tab; `true` once the daemon took the request. */
  newTab(): Promise<boolean>;
  switchTab(tabId: string): Promise<boolean>;
  /** Takes the active tab to the address the owner typed. */
  open(url: string): Promise<boolean>;
}

export function useHands(bot: Pick<Bot, "id">, state: BrowserState | null): Hands {
  const api = useApi();
  const t = useT().browser;
  const putBrowser = useApp((app) => app.putBrowser);
  const [busy, setBusy] = useState(false);
  const botId = bot.id;

  const take = useCallback(async () => {
    setBusy(true);
    await attempt(t.hands.takeFailed, async () => {
      putBrowser(await api.call("browser.take", { botId }));
    });
    setBusy(false);
  }, [api, botId, putBrowser, t.hands.takeFailed]);

  const release = useCallback(async () => {
    setBusy(true);
    await attempt(t.hands.giveBackFailed, async () => {
      putBrowser(await api.call("browser.release", { botId }));
    });
    setBusy(false);
  }, [api, botId, putBrowser, t.hands.giveBackFailed]);

  const send = useCallback(
    (input: BrowserInput) => {
      // One lost event is not worth a message; the page shows what arrived.
      api.call("browser.input", { botId, input }).catch(() => {});
    },
    [api, botId],
  );

  const newTab = useCallback(
    () => attempt(t.tabs.addFailed, () => api.call("browser.newTab", { botId })),
    [api, botId, t.tabs.addFailed],
  );
  const switchTab = useCallback(
    (tabId: string) =>
      attempt(t.tabs.switchFailed, () => api.call("browser.switchTab", { botId, tabId })),
    [api, botId, t.tabs.switchFailed],
  );
  const open = useCallback(
    (url: string) => attempt(t.goFailed, () => api.call("browser.open", { botId, url })),
    [api, botId, t.goFailed],
  );

  return {
    held: state?.status === "open" && state.control === "owner",
    busy,
    take,
    release,
    send,
    newTab,
    switchTab,
    open,
  };
}
