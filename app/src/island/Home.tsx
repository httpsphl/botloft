// The island's Home tab (spec 31.4): what waits for the owner, one at a
// time, then the agents at work, then today.

import { ChevronLeft, ChevronRight } from "lucide-react";
import { useState } from "react";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Bot, BotId, DayActivity } from "../lib/protocol.gen";
import { AskCard, RequestCard } from "./Cards";
import type { IslandHost } from "./host";
import { LiveAgents } from "./LiveAgents";
import { Today } from "./Today";
import type { Live } from "./useLive";
import type { Waiting } from "./useWaiting";

function Pager({ at, of, onMove }: { at: number; of: number; onMove: (to: number) => void }) {
  const t = useT();
  if (of < 2) {
    return null;
  }
  return (
    <span className="island-muted flex items-center gap-1 text-xs">
      <button
        type="button"
        aria-label={t.island.previous}
        disabled={at === 0}
        onClick={() => onMove(at - 1)}
      >
        <ChevronLeft size={14} />
      </button>
      {t.island.position(at + 1, of)}
      <button
        type="button"
        aria-label={t.island.next}
        disabled={at === of - 1}
        onClick={() => onMove(at + 1)}
      >
        <ChevronRight size={14} />
      </button>
    </span>
  );
}

function WaitingCard({
  api,
  host,
  waiting,
  refresh,
}: {
  api: BotloftApi;
  host: IslandHost;
  waiting: Waiting[];
  refresh: () => void;
}) {
  const [at, setAt] = useState(0);
  const index = Math.min(at, waiting.length - 1);
  const shown = waiting[index];
  if (!shown) {
    return null;
  }
  const pager = <Pager at={index} of={waiting.length} onMove={setAt} />;
  const open = () => host.openApp(shown.card.bot.id).catch(() => {});
  return shown.kind === "approval" ? (
    <RequestCard
      key={shown.id}
      api={api}
      card={shown.card}
      pager={pager}
      onOpen={open}
      onDone={refresh}
    />
  ) : (
    <AskCard
      key={shown.id}
      api={api}
      card={shown.card}
      pager={pager}
      onOpen={open}
      onDone={refresh}
    />
  );
}

export function Home({
  api,
  host,
  waiting,
  working,
  crews,
  live,
  today,
  refresh,
}: {
  api: BotloftApi;
  host: IslandHost;
  waiting: Waiting[];
  working: Bot[];
  crews: Map<string, string>;
  live: Map<BotId, Live>;
  today: DayActivity | null;
  refresh: () => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <WaitingCard api={api} host={host} waiting={waiting} refresh={refresh} />
      <LiveAgents working={working} crews={crews} live={live} />
      {today && <Today today={today} />}
    </div>
  );
}
