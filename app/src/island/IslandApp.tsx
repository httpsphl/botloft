// The island (spec 31): a pill at the top of the screen while agents work
// or wait for the owner, that grows into a panel with three tabs: Home,
// Chat and Usage. It comes down by itself when something new waits, peeks
// under the mouse, and takes files dropped on it to the Chat. The window
// always has the size of what it shows (spec 31.3), up to 60% of the
// screen's height; past that, the panel scrolls (31.2).

import { type DragEvent, useLayoutEffect, useRef, useState } from "react";
import { useFiles } from "../features/chat/useFiles";
import { useToday } from "../features/home/useToday";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { BotId } from "../lib/protocol.gen";
import { prefs, usePref } from "../shell/prefs";
import { ApiProvider, useApi } from "../store/context";
import { Bar, type Tab } from "./Bar";
import { ChatTab } from "./ChatTab";
import { Home } from "./Home";
import type { IslandHost } from "./host";
import { SHOWN } from "./LiveAgents";
import { CHEER_WIDTH, CheerPill, PILL, Pill } from "./Pill";
import { fiveHour, UsageTab, useUsage, WARN_AT } from "./UsageTab";
import { useAppear } from "./useAppear";
import { useCheer } from "./useCheer";
import { useLive } from "./useLive";
import { useWaiting } from "./useWaiting";

export { PILL } from "./Pill";
export const PANEL_WIDTH = 720;
/** The share of the screen's height the panel may take. */
const CAP = 0.6;

/** The panel's tallest height in CSS pixels; none where the screen has no size, as in jsdom. */
function panelCap(): number | undefined {
  const tall = window.screen.availHeight;
  return tall > 0 ? Math.floor(tall * CAP) : undefined;
}

const hasFiles = (event: DragEvent) => event.dataTransfer.types.includes("Files");

/** The island on its own connection, with no app store: the home's hooks read through it. */
export function IslandApp({ api, host }: { api: BotloftApi; host: IslandHost }) {
  return (
    <ApiProvider api={api}>
      <Island host={host} />
    </ApiProvider>
  );
}

function Island({ host }: { host: IslandHost }) {
  const t = useT();
  const api = useApi();
  const enabled = usePref(prefs.island) && host.supported;
  const screen = usePref(prefs.islandScreen);
  const { waiting, loaded, working, bots, crews, crewList, version, refresh } = useWaiting(api);
  const live = useLive(
    api,
    working.slice(0, SHOWN).map((bot) => bot.id),
  );
  const cheer = useCheer(api, bots, host);
  const today = useToday();
  const usage = useUsage(api, version);
  const files = useFiles();
  const [tab, setTab] = useState<Tab>("home");
  const { open, pinned, setOpen, hover } = useAppear({
    host,
    enabled,
    loaded,
    waitingIds: waiting.map((item) => item.id),
    onArrive: () => setTab("home"),
  });
  const [talkTo, setTalkTo] = useState<BotId | null>(null);
  const panel = useRef<HTMLDivElement>(null);
  const scroller = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(PILL.height);
  const cap = panelCap();
  // Pinned open, it stays until the owner hides it, even once nobody
  // works: they may be reading what an agent just answered.
  const visible =
    enabled &&
    ((open && pinned) ||
      waiting.length > 0 ||
      working.length > 0 ||
      cheer !== null ||
      files.files.length > 0);
  const expanded = open && visible;

  useLayoutEffect(() => {
    const content = panel.current;
    if (!expanded || !content) {
      setHeight(PILL.height);
      return;
    }
    // The content grows freely inside the scroller, so the observer keeps
    // seeing it; the window stops at the cap.
    const measure = () =>
      setHeight(Math.min(content.scrollHeight, panelCap() ?? Number.POSITIVE_INFINITY));
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
    (visible ? host.show(width, height, screen) : host.hide()).catch(() => {});
  }, [host, visible, width, height, screen]);

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
  // The three tabs share one scroller: each starts at its top.
  const showTab = (next: Tab) => {
    setTab(next);
    if (scroller.current) {
      scroller.current.scrollTop = 0;
    }
  };
  const color = waiting[0]?.card.bot.color ?? working[0]?.color ?? bots[0]?.color ?? "#FF7A59";
  const used = fiveHour(usage);
  return (
    <section
      aria-label={t.island.label}
      className="island"
      data-open={expanded}
      style={{ width, height }}
      {...hover}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          setOpen(false);
        }
      }}
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
          setOpen(true, true);
        }
      }}
    >
      {expanded ? (
        // At the panel's width from the first frame, so the shape grows
        // around the content instead of squeezing it.
        <div
          ref={scroller}
          className="island-content island-scroll"
          style={{ width: PANEL_WIDTH, maxHeight: cap }}
        >
          <div ref={panel} className="flex flex-col">
            <Bar
              api={api}
              crews={crewList}
              tab={tab}
              onTab={showTab}
              onOpenApp={() => openApp()}
              onHide={() => setOpen(false)}
              onChanged={refresh}
            />
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
        </div>
      ) : cheer ? (
        <CheerPill
          cheer={cheer}
          onOpen={() => {
            // To read the whole answer.
            setTalkTo(cheer.bot.id);
            setTab("chat");
            setOpen(true, true);
          }}
        />
      ) : (
        <Pill
          waiting={waiting.length}
          working={working.length}
          color={color}
          used={used >= WARN_AT ? used : null}
          onOpen={() => setOpen(true, true)}
        />
      )}
    </section>
  );
}
