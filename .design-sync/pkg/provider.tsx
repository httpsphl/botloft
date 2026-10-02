// What Botloft's screens need around them: the app's own providers (the
// daemon API, the app store, the native host), here backed by the app's
// in-memory fake daemon (FakeBotloft, the one `pnpm dev` and the tests use)
// instead of a running Botloft. Crews and bots passed in become the fake
// daemon's, so the sidebar, the crew view and the chat read them as the app
// would; a new array updates them, which lets a design animate states.

import { type ReactNode, useEffect, useRef, useState } from "react";
import type { ApprovalsAnswerParams, Bot, BotId, Crew } from "../../app/src/lib/protocol.gen";
import { FakeBotloft } from "../../app/src/lib/fake";
import { FakeHost } from "../../app/src/lib/fakeHost";
import { DaemonProvider, HostProvider, useApp, useAppStore } from "../../app/src/store/context";

export interface BotloftProviderProps {
  crews?: Crew[];
  bots?: Bot[];
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

export function BotloftProvider({ crews = [], bots = [], selectedBotId, onAnswer, children }: BotloftProviderProps) {
  const answer = useRef(onAnswer);
  answer.current = onAnswer;
  const [fake] = useState(() => {
    const api = new FakeBotloft();
    for (const crew of crews) api.crews.set(crew.id, crew);
    for (const bot of bots) api.bots.set(bot.id, bot);
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

  return (
    <HostProvider host={host}>
      <DaemonProvider api={fake}>
        <Selection botId={selectedBotId} />
        {children}
      </DaemonProvider>
    </HostProvider>
  );
}
