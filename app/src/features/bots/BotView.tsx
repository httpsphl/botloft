import { X } from "lucide-react";
import { useState } from "react";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { ChatView } from "../chat/ChatView";
import { BotHeader } from "./BotHeader";
import { stateView } from "./BotStateBadge";

/** A bot's conversation, with its details in a side panel (spec 15.1). */
export function BotView({ bot, crew }: { bot: Bot; crew: Crew }) {
  const [details, setDetails] = useState(false);
  const view = stateView(bot, crew.paused);

  return (
    <section aria-label={bot.name} className="flex min-h-0 flex-1 flex-col">
      <BotHeader
        bot={bot}
        crew={crew}
        detailsOpen={details}
        onToggleDetails={() => setDetails(!details)}
      />
      <Notices bot={bot} crew={crew} view={view} />
      <div className="flex min-h-0 flex-1">
        <ChatView bot={bot} stopped={bot.paused || crew.paused} />
        {details && <Details bot={bot} onClose={() => setDetails(false)} />}
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
  const notices = [];
  if (crew.paused && !bot.paused) {
    notices.push(
      <Callout key="crew" title={`${crew.name} is paused`}>
        Its bots stay stopped until you resume the crew.
      </Callout>,
    );
  }
  // An approval shows in the chat itself; the rest needs a word up here.
  if ((view.tone === "warn" || view.tone === "danger") && bot.state !== "needs_approval") {
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

function Details({ bot, onClose }: { bot: Bot; onClose(): void }) {
  return (
    <aside
      aria-label={`About ${bot.name}`}
      className="flex w-80 shrink-0 flex-col border-line border-l bg-panel"
    >
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">About {bot.name}</h2>
        <Button variant="ghost" size="sm" icon={X} label="Close details" onClick={onClose} />
      </header>
      <dl className="flex min-h-0 flex-col gap-4 overflow-y-auto p-4 text-sm">
        <div>
          <dt className="text-muted text-xs">Role</dt>
          <dd className="mt-0.5 text-ink-soft">{bot.role || "No role yet."}</dd>
        </div>
        <div>
          <dt className="text-muted text-xs">Folder</dt>
          <dd className="mt-0.5 break-all font-mono text-xs" data-selectable>
            {bot.workspace}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">Process</dt>
          <dd className="mt-0.5 font-mono text-xs">
            {bot.generation === null ? "not started" : `generation ${bot.generation}`}
          </dd>
        </div>
        <div>
          <dt className="text-muted text-xs">Instructions</dt>
          <dd className="mt-0.5 whitespace-pre-wrap text-ink-soft" data-selectable>
            {bot.instructions || "None yet."}
          </dd>
        </div>
      </dl>
    </aside>
  );
}
