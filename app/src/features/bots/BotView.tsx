import { useEffect, useState } from "react";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { Callout } from "../../ui/Callout";
import { Tabs, tabId } from "../../ui/Tabs";
import { TerminalView } from "../terminal/TerminalView";
import { BotHeader } from "./BotHeader";
import { stateView } from "./BotStateBadge";

type Pane = "terminal" | "details";

const TABS: { id: Pane; label: string }[] = [
  { id: "terminal", label: "Terminal" },
  { id: "details", label: "Details" },
];

/** How long a start takes before the trust prompt is the likely reason. */
const SLOW_START_MS = 4000;

/** True once the bot has been starting for a while. */
function useSlowStart(bot: Bot): boolean {
  const [slow, setSlow] = useState(false);
  // Each start of a process has its own generation.
  const launch = bot.state === "launching" ? bot.generation : null;
  useEffect(() => {
    setSlow(false);
    if (launch === null) {
      return;
    }
    const timer = setTimeout(() => setSlow(true), SLOW_START_MS);
    return () => clearTimeout(timer);
  }, [launch]);
  return slow && launch !== null;
}

export function BotView({ bot, crew }: { bot: Bot; crew: Crew }) {
  const [pane, setPane] = useState<Pane>("terminal");
  const slowStart = useSlowStart(bot);
  const view = stateView(bot, crew.paused);

  return (
    <section aria-label={bot.name} className="flex min-h-0 flex-1 flex-col">
      <BotHeader bot={bot} crew={crew} />
      <Notices bot={bot} crew={crew} slowStart={slowStart} view={view} />
      <Tabs<Pane> label="Bot views" tabs={TABS} value={pane} onChange={setPane} />
      {/* The terminal stays mounted so switching tabs keeps its screen. */}
      <div
        role="tabpanel"
        aria-labelledby={tabId("terminal")}
        className={pane === "terminal" ? "flex min-h-0 flex-1 flex-col" : "hidden"}
      >
        {bot.generation === null ? (
          <p className="p-6 text-muted text-sm">
            {bot.name} has not started since the daemon did.
            {(bot.paused || crew.paused) && " Resume it to start it."}
          </p>
        ) : (
          <TerminalView botId={bot.id} />
        )}
      </div>
      {pane === "details" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("details")}
          className="min-h-0 flex-1 overflow-y-auto p-5"
        >
          <Details bot={bot} />
        </div>
      )}
    </section>
  );
}

function Notices({
  bot,
  crew,
  slowStart,
  view,
}: {
  bot: Bot;
  crew: Crew;
  slowStart: boolean;
  view: ReturnType<typeof stateView>;
}) {
  const notices = [];
  if (crew.paused && !bot.paused) {
    notices.push(
      <Callout key="crew" title={`${crew.name} is paused`}>
        Its bots stay stopped until you resume the crew.
      </Callout>,
    );
  }
  if (slowStart) {
    notices.push(
      <Callout key="trust" title="Still starting">
        On a bot's first start, Claude Code asks whether you trust its folder. Choose "Yes, I trust
        this folder" in the terminal below; it asks only once.
      </Callout>,
    );
  }
  if (view.tone === "warn" || view.tone === "danger") {
    notices.push(
      <Callout key="state" tone={view.tone} title={view.label}>
        {view.hint}
      </Callout>,
    );
  }
  if (notices.length === 0) {
    return null;
  }
  return <div className="flex flex-col gap-2 border-line border-b px-5 py-3">{notices}</div>;
}

function Details({ bot }: { bot: Bot }) {
  return (
    <dl className="grid max-w-3xl grid-cols-[9rem_1fr] gap-x-4 gap-y-3 border border-line bg-panel p-4 text-sm">
      <dt className="text-muted">Role</dt>
      <dd className="text-ink-soft">{bot.role || "No role yet."}</dd>
      <dt className="text-muted">Folder</dt>
      <dd className="break-all font-mono text-xs" data-selectable>
        {bot.workspace}
      </dd>
      <dt className="text-muted">Process</dt>
      <dd className="font-mono text-xs">
        {bot.generation === null ? "not started" : `generation ${bot.generation}`}
      </dd>
      <dt className="text-muted">Instructions</dt>
      <dd className="whitespace-pre-wrap text-ink-soft" data-selectable>
        {bot.instructions || "None yet."}
      </dd>
    </dl>
  );
}
