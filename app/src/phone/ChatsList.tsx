// The list of conversations (spec 28.12): the bots by crew, each with its
// mascot, its last activity, "working…" while it is busy and a dot when it
// wrote since the owner looked. Chips at the top narrow it to one crew, as
// the tabs of the computer do.

import { useT } from "../i18n";
import type { ChatLine } from "../lib/protocol.gen";
import { BotDot } from "./parts";

export function isUnread(line: ChatLine, seen: Record<string, number>): boolean {
  return line.lastReplyAt !== undefined && line.lastReplyAt > (seen[line.botId] ?? 0);
}

/** The crews of the list, by name. */
export function crewsOf(chats: ChatLine[]): string[] {
  return [...new Set(chats.map((line) => line.crew))].sort((a, b) => a.localeCompare(b));
}

const Dot = ({ label }: { label: string }) => (
  <span role="img" aria-label={label} className="h-3 w-3 shrink-0 rounded-full bg-work" />
);

export function CrewChips({
  chats,
  seen,
  crew,
  pick,
}: {
  chats: ChatLine[];
  seen: Record<string, number>;
  crew: string | null;
  pick(crew: string | null): void;
}) {
  const t = useT().phone;
  const crews = crewsOf(chats);
  if (crews.length < 2) {
    return null;
  }
  const chip = (name: string | null, label: string) => {
    const on = crew === name;
    const news = chats.some(
      (line) => (name === null || line.crew === name) && isUnread(line, seen),
    );
    return (
      <button
        key={label}
        type="button"
        aria-pressed={on}
        onClick={() => pick(name)}
        className={`inline-flex h-10 shrink-0 items-center gap-2 rounded-full border px-4 font-medium text-sm ${on ? "border-ink bg-ink text-canvas" : "border-line-strong bg-panel text-ink"}`}
      >
        {label}
        {news && <Dot label={t.tabs.newReply} />}
      </button>
    );
  };
  return (
    <fieldset
      aria-label={t.chats.crewsLabel}
      className="-mx-4 flex min-w-0 gap-2 overflow-x-auto border-0 px-4 pb-1"
    >
      {chip(null, t.chats.allCrews)}
      {crews.map((name) => chip(name, name))}
    </fieldset>
  );
}

export function ChatsList({
  chats,
  loaded,
  seen,
  crew,
  open,
}: {
  chats: ChatLine[];
  loaded: boolean;
  seen: Record<string, number>;
  crew: string | null;
  open(botId: string): void;
}) {
  const t = useT().phone;
  if (chats.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-16 text-center">
        {loaded ? (
          <>
            <p className="font-medium text-base">{t.chats.empty}</p>
            <p className="text-muted text-sm">{t.chats.emptyBody}</p>
          </>
        ) : (
          <p className="text-muted text-sm">{t.chats.loading}</p>
        )}
      </div>
    );
  }
  const shown = crew === null ? chats : chats.filter((line) => line.crew === crew);
  const groups = crewsOf(shown).map((name) => ({
    name,
    lines: shown.filter((line) => line.crew === name),
  }));
  const headings = crew === null && groups.length > 1;
  return (
    <div className="flex flex-col gap-5">
      {groups.map((group) => (
        <section key={group.name} aria-label={group.name} className="flex flex-col gap-2">
          {headings && (
            <h2 className="flex items-baseline gap-2 px-1 font-semibold text-muted text-xs uppercase tracking-[0.12em]">
              {group.name}
              <span className="font-normal normal-case tracking-normal">
                {t.chats.bots(group.lines.length)}
              </span>
            </h2>
          )}
          <ul className="flex flex-col gap-2">
            {group.lines.map((line) => (
              <li key={line.botId}>
                <ChatRow
                  line={line}
                  unread={isUnread(line, seen)}
                  open={() => open(line.botId)}
                  showCrew={!headings && crew === null}
                />
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

function ChatRow({
  line,
  unread,
  open,
  showCrew,
}: {
  line: ChatLine;
  unread: boolean;
  open(): void;
  showCrew: boolean;
}) {
  const t = useT().phone;
  const busy = line.state === "busy" || line.state === "needs_approval";
  const kind = line.last ? t.chats.kind[line.last.kind] : "";
  const last = line.last ? `${kind ? `${kind}: ` : ""}${line.last.text}` : "";
  return (
    <button
      type="button"
      onClick={open}
      className="flex min-h-[4.5rem] w-full items-center gap-3 rounded-2xl border border-line bg-panel p-3 text-left active:scale-[0.99]"
    >
      <BotDot name={line.name} color={line.color} state={line.state} size={44} />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="flex items-center gap-2">
          <span className="truncate font-medium">{line.name}</span>
          {showCrew && (
            <span className="truncate text-muted text-xs">{t.chats.inCrew(line.crew)}</span>
          )}
        </span>
        <span className="truncate text-ink-soft text-sm">{busy ? t.chats.working : last}</span>
      </span>
      {unread && <Dot label={t.tabs.newReply} />}
    </button>
  );
}
