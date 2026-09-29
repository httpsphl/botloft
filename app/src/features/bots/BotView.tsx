import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { routinesOf } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { ChatView } from "../chat/ChatView";
import { FilesPanel } from "../files/FilesPanel";
import { ShowFile } from "../files/showFile";
import { useBotFiles } from "../files/useBotFiles";
import { SignInButton } from "../onboarding/SignIn";
import { BotRoutines } from "../routines/RoutineList";
import { BotHeader } from "./BotHeader";
import { stateView } from "./BotStateBadge";

type Pane = "chat" | "routines";
/** What the panel beside the chat shows. */
type Side = "details" | "files" | null;

/**
 * A bot's conversation and its routines, with its details in a side panel
 * (spec 15.1, 20.9).
 */
export function BotView({ bot, crew }: { bot: Bot; crew: Crew }) {
  const t = useT();
  const [side, setSide] = useState<Side>(null);
  const [pane, setPane] = useState<Pane>("chat");
  const files = useBotFiles(bot);
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
  const showFile = async (path: string) => {
    if (!filesOpen) {
      seen();
    }
    setSide("files");
    await files.refresh();
    setShown(path);
  };
  const fresh = filesOpen ? 0 : files.files.filter((file) => file.modifiedAt > seenAt).length;
  const routines = useApp(useShallow((state) => routinesOf(state, bot.id)));
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
      />
      <Notices bot={bot} crew={crew} view={view} />
      <Tabs<Pane> label={bot.name} tabs={tabs} value={pane} onChange={setPane} />
      <div className="flex min-h-0 flex-1">
        <div role="tabpanel" aria-labelledby={tabId(pane)} className="flex min-h-0 min-w-0 flex-1">
          {pane === "chat" ? (
            <ShowFile.Provider value={showFile}>
              <ChatView bot={bot} stopped={bot.paused || crew.paused} />
            </ShowFile.Provider>
          ) : (
            <BotRoutines bot={bot} />
          )}
        </div>
        {side === "details" && <Details bot={bot} onClose={() => setSide(null)} />}
        {filesOpen && (
          <FilesPanel
            bot={bot}
            data={files}
            since={since}
            path={shown}
            onPath={setShown}
            onClose={toggleFiles}
          />
        )}
      </div>
    </section>
  );
}

function Notices({
  bot,
  crew,
  view,
}: {
  bot: Bot;
  crew: Crew;
  view: ReturnType<typeof stateView>;
}) {
  const t = useT();
  const notices = [];
  if (crew.paused && !bot.paused) {
    notices.push(
      <Callout key="crew" title={t.bots.notices.crewPaused(crew.name)}>
        {t.bots.notices.crewPausedBody}
      </Callout>,
    );
  }
  // An approval shows in the chat itself; the rest needs a word up here.
  if ((view.tone === "warn" || view.tone === "danger") && bot.state !== "needs_approval") {
    notices.push(
      <Callout key="state" tone={view.tone} title={view.label}>
        {view.hint}
        {bot.state === "auth_error" && <SignInButton />}
      </Callout>,
    );
  }
  if (notices.length === 0) {
    return null;
  }
  return <div className="flex flex-col gap-2 border-line border-b px-5 py-3">{notices}</div>;
}

function Details({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const words = useT().bots.details;
  return (
    <aside
      aria-label={words.title(bot.name)}
      className="flex w-80 shrink-0 flex-col border-line border-l bg-panel"
    >
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">{words.title(bot.name)}</h2>
        <Button variant="ghost" size="sm" icon={X} label={words.close} onClick={onClose} />
      </header>
      <dl className="flex min-h-0 flex-col gap-4 overflow-y-auto p-4 text-sm">
        <div>
          <dt className="text-muted text-xs">{words.role}</dt>
          <dd className="mt-0.5 text-ink-soft">{bot.role || words.noRole}</dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.folder}</dt>
          <dd className="mt-0.5 break-all font-mono text-xs" data-selectable>
            {bot.workspace}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.process}</dt>
          <dd className="mt-0.5 font-mono text-xs">
            {bot.generation === null ? words.notStarted : words.generation(bot.generation)}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">{words.instructions}</dt>
          <dd className="mt-0.5 whitespace-pre-wrap text-ink-soft" data-selectable>
            {bot.instructions || words.noInstructions}
          </dd>
        </div>
      </dl>
    </aside>
  );
}
