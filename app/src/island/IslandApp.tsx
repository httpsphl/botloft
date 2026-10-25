// The island (spec 31): a pill at the top of the screen while agents work
// or wait for the owner, that grows into a panel with three tabs: Home,
// Chat and Usage. Files dropped on it go to the Chat. The window always has
// the size of what it shows (spec 31.3).

import { Gauge, House, MessageCircle, SquareArrowOutUpRight, X } from "lucide-react";
import { type DragEvent, type ReactNode, useLayoutEffect, useRef, useState } from "react";
import { useFiles } from "../features/chat/useFiles";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { BotId } from "../lib/protocol.gen";
import { ChatTab } from "./ChatTab";
import { Home } from "./Home";
import type { IslandHost } from "./host";
import { SHOWN } from "./LiveAgents";
import { CHEER_WIDTH, CheerPill, PILL, Pill } from "./Pill";
import { useToday } from "./Today";
import { fiveHour, UsageTab, useUsage, WARN_AT } from "./UsageTab";
import { useCheer } from "./useCheer";
import { useLive } from "./useLive";
import { useWaiting } from "./useWaiting";

export { PILL } from "./Pill";
export const PANEL_WIDTH = 720;

type Tab = "home" | "chat" | "usage";

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

export function IslandApp({ api, host }: { api: BotloftApi; host: IslandHost }) {
  const t = useT();
  const { waiting, working, bots, crews, version, refresh } = useWaiting(api);
  const live = useLive(
    api,
    working.slice(0, SHOWN).map((bot) => bot.id),
  );
  const cheer = useCheer(api, bots);
  const today = useToday(api, version);
  const usage = useUsage(api, version);
  const files = useFiles();
  const [open, setOpen] = useState(false);
  const [tab, setTab] = useState<Tab>("home");
  const [talkTo, setTalkTo] = useState<BotId | null>(null);
  const panel = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(PILL.height);
  const visible =
    waiting.length > 0 || working.length > 0 || cheer !== null || files.files.length > 0;
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
  // The agent the Chat starts on: who waits, then who works, then the first.
  const talking = talkTo ?? waiting[0]?.card.bot.id ?? working[0]?.id ?? bots[0]?.id ?? null;
  const ordered = [
    ...bots.filter((bot) => bot.state === "needs_approval" || bot.state === "busy"),
    ...bots.filter((bot) => bot.state !== "needs_approval" && bot.state !== "busy"),
  ];
  const openApp = (botId?: BotId) => host.openApp(botId).catch(() => {});
  const tabs: [Tab, string, ReactNode][] = [
    ["home", t.island.tabs.home, <House key="home" aria-hidden size={16} />],
    ["chat", t.island.tabs.chat, <MessageCircle key="chat" aria-hidden size={16} />],
    ["usage", t.island.tabs.usage, <Gauge key="usage" aria-hidden size={16} />],
  ];
  const color = waiting[0]?.card.bot.color ?? working[0]?.color ?? bots[0]?.color ?? "#FF7A59";
  const used = fiveHour(usage);
  return (
    <section
      aria-label={t.island.label}
      className="island"
      data-open={expanded}
      style={{ width, height }}
      onDragOver={(event) => {
        if (hasFiles(event)) {
          event.preventDefault();
        }
      }}
      onDrop={(event) => {
        if (hasFiles(event)) {
          event.preventDefault();
          void files.add([...event.dataTransfer.files]);
          setTab("chat");
          setOpen(true);
        }
      }}
    >
      {expanded ? (
        <div ref={panel} className="island-content flex flex-col">
          <div className="flex items-center justify-between px-3 pt-2.5 pb-2">
            <div role="tablist" aria-label={t.island.tabs.label} className="flex gap-1">
              {tabs.map(([id, name, icon]) => (
                <button
                  key={id}
                  type="button"
                  role="tab"
                  aria-selected={tab === id}
                  aria-label={name}
                  title={name}
                  className="island-tab"
                  onClick={() => setTab(id)}
                >
                  {icon}
                </button>
              ))}
            </div>
            <div className="flex gap-1">
              <button
                type="button"
                className="island-tab"
                title={t.island.openApp}
                aria-label={t.island.openApp}
                onClick={() => openApp()}
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
            {tab === "home" && (
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
            )}
            {tab === "chat" && (
              <ChatTab
                api={api}
                bots={ordered}
                botId={talking}
                onPick={setTalkTo}
                files={files.files}
                onRemoveFile={files.remove}
                fileError={files.error}
                onSent={files.clear}
                onOpen={openApp}
              />
            )}
            {tab === "usage" && <UsageTab api={api} usage={usage} bots={bots} />}
          </div>
        </div>
      ) : cheer ? (
        <CheerPill cheer={cheer} onOpen={() => setOpen(true)} />
      ) : (
        <Pill
          waiting={waiting.length}
          working={working.length}
          color={color}
          used={used >= WARN_AT ? used : null}
          onOpen={() => setOpen(true)}
        />
      )}
    </section>
  );
}
