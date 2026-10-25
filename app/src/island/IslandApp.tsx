// The island (spec 31): a pill at the top of the screen while agents work
// or wait for the owner, that grows into a panel: what waits first, then
// who works, then today. The window always has the size of what it shows
// (spec 31.3).

import { ChevronLeft, ChevronRight, House, SquareArrowOutUpRight, X } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Bot, BotId, DayActivity } from "../lib/protocol.gen";
import { AskCard, RequestCard } from "./Cards";
import type { IslandHost } from "./host";
import { LiveAgents, SHOWN } from "./LiveAgents";
import { CHEER_WIDTH, CheerPill, PILL, Pill } from "./Pill";
import { Today, useToday } from "./Today";
import { useCheer } from "./useCheer";
import { type Live, useLive } from "./useLive";
import { useWaiting, type Waiting } from "./useWaiting";

export { PILL } from "./Pill";
export const PANEL_WIDTH = 720;

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

function Home({
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

export function IslandApp({ api, host }: { api: BotloftApi; host: IslandHost }) {
  const t = useT();
  const { waiting, working, bots, crews, version, refresh } = useWaiting(api);
  const live = useLive(
    api,
    working.slice(0, SHOWN).map((bot) => bot.id),
  );
  const cheer = useCheer(api, bots);
  const today = useToday(api, version);
  const [open, setOpen] = useState(false);
  const panel = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(PILL.height);
  const visible = waiting.length > 0 || working.length > 0 || cheer !== null;
  const expanded = open && visible;

  useLayoutEffect(() => {
    const content = panel.current;
    if (!expanded || !content) {
      setHeight(PILL.height);
      return;
    }
    const measure = () => setHeight(content.scrollHeight);
    measure();
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    const watch = new ResizeObserver(measure);
    watch.observe(content);
    return () => watch.disconnect();
  }, [expanded]);

  const width = expanded ? PANEL_WIDTH : cheer ? CHEER_WIDTH : PILL.width;
  useLayoutEffect(() => {
    (visible ? host.show(width, height) : host.hide()).catch(() => {});
  }, [host, visible, width, height]);

  if (!visible) {
    return null;
  }
  const color = waiting[0]?.card.bot.color ?? working[0]?.color ?? "#FF7A59";
  return (
    <section
      aria-label={t.island.label}
      className="island"
      data-open={expanded}
      style={{ width, height }}
    >
      {expanded ? (
        <div ref={panel} className="island-content flex flex-col">
          <div className="flex items-center justify-between px-3 pt-2.5 pb-2">
            <div role="tablist" aria-label={t.island.tabs.label} className="flex gap-1">
              <button
                type="button"
                role="tab"
                aria-selected
                className="island-tab"
                title={t.island.tabs.home}
              >
                <House aria-label={t.island.tabs.home} size={16} />
              </button>
            </div>
            <div className="flex gap-1">
              <button
                type="button"
                className="island-tab"
                title={t.island.openApp}
                aria-label={t.island.openApp}
                onClick={() => host.openApp().catch(() => {})}
              >
                <SquareArrowOutUpRight aria-hidden size={15} />
              </button>
              <button
                type="button"
                className="island-tab"
                title={t.island.hide}
                aria-label={t.island.hide}
                onClick={() => setOpen(false)}
              >
                <X aria-hidden size={15} />
              </button>
            </div>
          </div>
          <div className="px-3 pb-3">
            <Home
              api={api}
              host={host}
              waiting={waiting}
              working={working}
              crews={crews}
              live={live}
              today={today}
              refresh={refresh}
            />
          </div>
        </div>
      ) : cheer ? (
        <CheerPill cheer={cheer} onOpen={() => setOpen(true)} />
      ) : (
        <Pill
          waiting={waiting.length}
          working={working.length}
          color={color}
          onOpen={() => setOpen(true)}
        />
      )}
    </section>
  );
}
