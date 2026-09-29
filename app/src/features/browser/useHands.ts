// The owner's hands in a bot's browser (spec 21.10): taking it, giving it
// back, and sending what they do on the page. Events go out in order and
// without waiting for each other: the daemon queues them for the page.

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
}

export function useHands(bot: Pick<Bot, "id">, state: BrowserState | null): Hands {
  const api = useApi();
  const t = useT().browser.hands;
  const putBrowser = useApp((app) => app.putBrowser);
  const [busy, setBusy] = useState(false);
  const botId = bot.id;

  const take = useCallback(async () => {
    setBusy(true);
    await attempt(t.takeFailed, async () => {
      putBrowser(await api.call("browser.take", { botId }));
    });
    setBusy(false);
  }, [api, botId, putBrowser, t.takeFailed]);

  const release = useCallback(async () => {
    setBusy(true);
    await attempt(t.giveBackFailed, async () => {
      putBrowser(await api.call("browser.release", { botId }));
    });
    setBusy(false);
  }, [api, botId, putBrowser, t.giveBackFailed]);

  const send = useCallback(
    (input: BrowserInput) => {
      // One lost event is not worth a message; the page shows what arrived.
      api.call("browser.input", { botId, input }).catch(() => {});
    },
    [api, botId],
  );

  return {
    held: state?.status === "open" && state.control === "owner",
    busy,
    take,
    release,
    send,
  };
}
