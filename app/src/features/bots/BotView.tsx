import { useEffect, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, BotFile, Crew } from "../../lib/protocol.gen";
import { useWindowVisible } from "../../shell/visibility";
import { routinesOf } from "../../store/app";
import { useApp } from "../../store/context";
import { PanelClosing, PanelRestored } from "../../ui/panelMotion";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { useStable } from "../../ui/useStable";
import { BrowserPanel } from "../browser/BrowserPanel";
import { ShowBrowser } from "../browser/showBrowser";
import { ChatView } from "../chat/ChatView";
import { FilesPanel } from "../files/FilesPanel";
import { ShowFile } from "../files/showFile";
import { useBotFiles } from "../files/useBotFiles";
import { BotRoutines } from "../routines/RoutineList";
import { ScreensPanel } from "../screens/ScreensPanel";
import { ShowScreen } from "../screens/showScreen";
import { useScreens } from "../screens/useScreens";
import { Dock } from "../terminal/ComputerDock";
import { ShowTerminal } from "../terminal/showTerminal";
import { TerminalPanel } from "../terminal/TerminalPanel";
import { BotHeader } from "./BotHeader";
import { Details, Notices } from "./BotPanels";
import { stateView } from "./BotStateBadge";
import { useFollowBot } from "./useFollowBot";
import { usePanel } from "./usePanel";

type Pane = "chat" | "routines";

/**
 * A bot's conversation and its routines, with its details in a side panel
 * (spec 15.1, 20.9).
 */
export function BotView({ bot, crew }: { bot: Bot; crew: Crew }) {
  const t = useT();
  // Kept per bot in the store: the panel the owner left open is back when
  // they come back to the bot, and one they closed stays closed (spec 15.1).
  const { side, setSide, beside, restored, closing } = usePanel(bot.id);
  const [pane, setPane] = useState<Pane>("chat");
  const files = useBotFiles(bot, side === "files");
  // What the owner has seen: files newer than this are new to them.
  const [seenAt, setSeenAt] = useState(() => Date.now());
  const [since, setSince] = useState(seenAt);
  // biome-ignore lint/correctness/useExhaustiveDependencies: a new bot has its own files to see
  useEffect(() => {
    const now = Date.now();
    setSeenAt(now);
    setSince(now);
  }, [bot.id]);
  const [shown, setShown] = useState<string | null>(null);
  // A file shared in the chat, for when the panel's list leaves it out.
  const [described, setDescribed] = useState<BotFile | null>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: another bot's file is not this bot's
  useEffect(() => setShown(null), [bot.id]);
  const filesOpen = side === "files";
  const seen = () => {
    // What was there when the panel opened or closed counts as seen.
    setSince(seenAt);
    setSeenAt(Date.now());
  };
  const toggleFiles = () => {
    seen();
    setShown(null);
    setSide(filesOpen ? null : "files");
  };
  // A tool line in the chat points to its file: read the list first, so the
  // file is in it when the panel goes to it.
  // Stable, so the chat does not re-render when this view does.
  const showFile = useStable(async (path: string, file?: BotFile) => {
    if (!filesOpen) {
      seen();
    }
    setSide("files");
    await files.refresh();
    setDescribed(file ?? null);
    setShown(path);
  });
  const fresh = filesOpen ? 0 : files.files.filter((file) => file.modifiedAt > seenAt).length;
  const routines = useApp(useShallow((state) => routinesOf(state, bot.id)));
  // While the owner can see the chat, each new reply is seen (spec 15.1).
  const visible = useWindowVisible();
  const replyAt = useApp((state) => state.replyAt[bot.id]);
  const markSeen = useApp((state) => state.markSeen);
  useEffect(() => {
    if (visible && replyAt !== undefined) {
      markSeen(bot.id);
    }
  }, [bot.id, visible, replyAt, markSeen]);
  // A browser at rest is open, but the bot is not using it (spec 21.2).
  const browsing = useApp((state) => {
    const browser = state.browsers[bot.id];
    return browser?.status === "starting" || (browser?.status === "open" && !browser.resting);
  });
  const browserOpen = side === "browser";
  const asking = useApp((state) => Boolean(state.browsers[bot.id]?.ask));
  // Each ask to open the browser in the owner's hands counts up (spec 21.10).
  const [takeBrowser, setTakeBrowser] = useState(0);
  const screens = useScreens(bot);
  const screensOpen = side === "screens";
  const [screenPath, setScreenPath] = useState<string | null>(null);
  const showScreen = useStable((path: string | null) => {
    if (filesOpen) {
      seen();
    }
    setSide("screens");
    setScreenPath(path);
    void screens.refresh();
  });
  // biome-ignore lint/correctness/useExhaustiveDependencies: another bot's screen is not this bot's
  useEffect(() => setScreenPath(null), [bot.id]);
  const showBrowser = useStable((options?: { take?: boolean }) => {
    if (filesOpen) {
      seen();
    }
    setSide("browser");
    if (options?.take) {
      setTakeBrowser((count) => count + 1);
    }
  });
  // The panel follows what the bot starts doing (spec 15.1), over the one
  // that came back too.
  useFollowBot({
    bot,
    side,
    writing: screens.writing,
    open: (panel) => (panel === "browser" ? showBrowser() : showScreen(null)),
    close: (panel) => side === panel && setSide(null),
  });
  const showTerminal = useStable(() => {
    if (filesOpen) {
      seen();
    }
    setSide("terminal");
  });
  // The dock of the bot's computer: browser, terminal and files.
  const dock = {
    open: side,
    pick: (place: "browser" | "terminal" | "files") => {
      if (place === "browser") {
        showBrowser();
      } else if (place === "terminal") {
        showTerminal();
      } else if (!filesOpen) {
        toggleFiles();
      }
    },
  };
  const view = stateView(bot, crew.paused, t);
  const tabs: Tab<Pane>[] = [
    { id: "chat", label: t.routines.chatTab },
    {
      id: "routines",
      label: routines.length > 0 ? `${t.routines.tab} (${routines.length})` : t.routines.tab,
    },
  ];

  return (
    <section aria-label={bot.name} className="flex min-h-0 flex-1 flex-col">
      <BotHeader
        bot={bot}
        crew={crew}
        detailsOpen={side === "details"}
        onToggleDetails={() => setSide(side === "details" ? null : "details")}
        filesOpen={filesOpen}
        freshFiles={fresh}
        onToggleFiles={toggleFiles}
        browserOpen={browserOpen}
        browsing={browsing}
        asking={asking}
        onToggleBrowser={() => (browserOpen ? setSide(null) : showBrowser())}
        screensOpen={screensOpen}
        drawing={screens.writing !== null}
        onToggleScreens={() => (screensOpen ? setSide(null) : showScreen(null))}
      />
      <Notices bot={bot} crew={crew} view={view} />
      <Tabs<Pane> label={bot.name} tabs={tabs} value={pane} onChange={setPane} />
      {/* A panel slides out over the chat, and is cut at the window's edge. */}
      <div className="relative flex min-h-0 flex-1 overflow-x-clip">
        <div role="tabpanel" aria-labelledby={tabId(pane)} className="flex min-h-0 min-w-0 flex-1">
          {pane === "chat" ? (
            <ShowFile.Provider value={showFile}>
              <ShowBrowser.Provider value={showBrowser}>
                <ShowScreen.Provider value={showScreen}>
                  <ShowTerminal.Provider value={showTerminal}>
                    <ChatView bot={bot} stopped={bot.paused || crew.paused} />
                  </ShowTerminal.Provider>
                </ShowScreen.Provider>
              </ShowBrowser.Provider>
            </ShowFile.Provider>
          ) : (
            <BotRoutines bot={bot} />
          )}
        </div>
        <Dock.Provider value={dock}>
          <PanelRestored.Provider value={restored}>
            <PanelClosing.Provider value={closing}>
              {beside === "details" && <Details bot={bot} onClose={() => setSide(null)} />}
              {beside === "browser" && (
                <BrowserPanel bot={bot} take={takeBrowser} onClose={() => setSide(null)} />
              )}
              {beside === "screens" && (
                <ScreensPanel
                  bot={bot}
                  data={screens}
                  path={screenPath}
                  onPath={setScreenPath}
                  onClose={() => setSide(null)}
                />
              )}
              {beside === "files" && (
                <FilesPanel
                  bot={bot}
                  data={files}
                  since={since}
                  path={shown}
                  described={described}
                  onPath={setShown}
                  onClose={toggleFiles}
                />
              )}
              {beside === "terminal" && <TerminalPanel bot={bot} onClose={() => setSide(null)} />}
            </PanelClosing.Provider>
          </PanelRestored.Provider>
        </Dock.Provider>
      </div>
    </section>
  );
}
