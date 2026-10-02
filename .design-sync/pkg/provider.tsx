// What Botloft's screens need around them: the app's own providers (the
// daemon API, the app store, the native host), here backed by the app's
// in-memory fake daemon (FakeBotloft, the one `pnpm dev` and the tests use)
// instead of a running Botloft. Crews and bots passed in become the fake
// daemon's, so the sidebar, the crew view and the chat read them as the app
// would; a new array updates them, which lets a design animate states.
// A bot's open browser is drawn by the same page painter as the app's dev
// preview (app/src/dev/seedPage.ts) and streamed to BrowserPanel as frames.

import { type ReactNode, useEffect, useRef, useState } from "react";
import { activeTab, drawPage } from "../../app/src/dev/seedPage";
import { FakeBotloft } from "../../app/src/lib/fake";
import { FakeHost } from "../../app/src/lib/fakeHost";
import type {
  ApprovalsAnswerParams,
  Bot,
  BotId,
  BrowserActionKind,
  Crew,
} from "../../app/src/lib/protocol.gen";
import { DaemonProvider, HostProvider, useApp, useAppStore } from "../../app/src/store/context";

/** A bot's browser, open on `url`; earlier tabs stay behind it. */
export interface BrowserSeed {
  bot: Bot;
  url: string;
  title: string;
  tabs?: { url: string; title: string }[];
  /** What the bot just did on the page, shown by its cursor; a new object plays it again. */
  action?: { kind: BrowserActionKind; x?: number; y?: number; label?: string };
}

export interface BotloftProviderProps {
  crews?: Crew[];
  bots?: Bot[];
  /** Open browsers, shown live by BrowserPanel. */
  browsers?: BrowserSeed[];
  /** The bot whose chat is open: highlighted in the sidebar. */
  selectedBotId?: BotId | null;
  /** Called when Allow or Deny is clicked on a request card. */
  onAnswer?: (answer: { approvalId: string; allow: boolean; note?: string }) => void;
  children: ReactNode;
}

function Selection({ botId }: { botId: BotId | null | undefined }) {
  const store = useAppStore();
  const loaded = useApp((state) => state.loaded);
  useEffect(() => {
    if (loaded && botId) {
      store.getState().selectBot(botId);
    }
  }, [store, loaded, botId]);
  return null;
}

/** Opens (or moves) the bot's browser, draws its page and plays its action. */
function showBrowser(fake: FakeBotloft, browser: BrowserSeed): void {
  const id = browser.bot.id;
  if (fake.browser.state(id).status === "open") {
    fake.browser.open(id, browser.url, browser.title);
  } else {
    for (const tab of browser.tabs ?? []) {
      fake.browser.openTab(id, tab.url, tab.title);
    }
    // The page the bot is on: the last tab, the active one.
    fake.browser.openTab(id, browser.url, browser.title);
    fake.browser.open(id, browser.url, browser.title);
  }
  fake.browser.paint(id, (size) => drawPage(activeTab(fake, id), size));
  if (browser.action) {
    fake.browser.act(id, browser.action.kind, browser.action);
  }
}

export function BotloftProvider({
  crews = [],
  bots = [],
  browsers = [],
  selectedBotId,
  onAnswer,
  children,
}: BotloftProviderProps) {
  const answer = useRef(onAnswer);
  answer.current = onAnswer;
  const [fake] = useState(() => {
    const api = new FakeBotloft();
    for (const crew of crews) api.crews.set(crew.id, crew);
    for (const bot of bots) api.bots.set(bot.id, bot);
    // Before the store connects: the fake answers its first `browser.list`
    // at once, and an answer from before the browsers opened would win.
    for (const browser of browsers) showBrowser(api, browser);
    const call = api.call.bind(api) as FakeBotloft["call"];
    // A request card's answer goes to the design, not to a daemon.
    api.call = ((method: string, ...params: unknown[]) => {
      if (method === "approvals.answer") {
        const asked = params[0] as ApprovalsAnswerParams;
        answer.current?.({
          approvalId: asked.approvalId,
          allow: asked.allow,
          ...(asked.note ? { note: asked.note } : {}),
        });
        return Promise.resolve({});
      }
      return (call as (method: string, ...params: unknown[]) => Promise<unknown>)(method, ...params);
    }) as FakeBotloft["call"];
    return api;
  });
  const [host] = useState(() => new FakeHost());

  useEffect(() => {
    for (const crew of crews) {
      fake.crews.set(crew.id, crew);
      fake.changedCrew(crew);
    }
  }, [fake, crews]);
  useEffect(() => {
    for (const bot of bots) {
      fake.bots.set(bot.id, bot);
      fake.changedBot(bot);
    }
  }, [fake, bots]);
  useEffect(() => {
    for (const browser of browsers) {
      showBrowser(fake, browser);
    }
  }, [fake, browsers]);

  return (
    <HostProvider host={host}>
      <DaemonProvider api={fake}>
        <Selection botId={selectedBotId} />
        {children}
      </DaemonProvider>
    </HostProvider>
  );
}
