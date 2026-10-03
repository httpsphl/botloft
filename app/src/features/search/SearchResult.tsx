// One search result: whose chat, who wrote it, when, and the words around
// what was found, highlighted. A click opens the chat there.

import { useT } from "../../i18n";
import { when } from "../../lib/format";
import { type SearchHit, SNIPPET_MARKS } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";

/** Text without the markdown marks a bot writes (bold, code, headings, quotes). */
function plain(text: string): string {
  return text.replace(/\*+|__|~~|`+/g, "").replace(/(^|\s)(#{1,6}|>)\s/g, "$1");
}

/** The snippet as text and highlighted parts, in order, without markdown marks. */
export function snippetParts(snippet: string): { text: string; found: boolean }[] {
  const parts: { text: string; found: boolean }[] = [];
  for (const piece of snippet.split(SNIPPET_MARKS.open)) {
    const close = piece.indexOf(SNIPPET_MARKS.close);
    if (close < 0) {
      parts.push({ text: plain(piece), found: false });
    } else {
      parts.push({ text: plain(piece.slice(0, close)), found: true });
      parts.push({ text: plain(piece.slice(close + 1)), found: false });
    }
  }
  return parts.filter((part) => part.text !== "");
}

export function SearchResult({ hit }: { hit: SearchHit }) {
  const t = useT();
  const words = t.search;
  const bot = useApp((state) => state.bots[hit.item.botId]);
  const crew = useApp((state) => state.crews[hit.crewId]);
  const sender = useApp((state) => {
    const body = hit.item.body;
    return body.kind === "inbound" && body.message.fromBotId
      ? (state.bots[body.message.fromBotId]?.name ?? null)
      : null;
  });
  const openAt = useApp((state) => state.openAt);
  if (!bot) {
    return null;
  }
  const body = hit.item.body;
  let author = bot.name;
  if (body.kind === "inbound") {
    const from = body.message.fromKind;
    author =
      from === "owner" ? words.you : from === "system" ? words.botloft : (sender ?? words.goneBot);
  }
  return (
    <li>
      <button
        type="button"
        title={words.openHere(bot.name)}
        onClick={() => openAt(bot.id, hit.item.id)}
        className="flex w-full gap-3 rounded-xl px-3 py-2.5 text-left transition-colors duration-150 hover:bg-sunken"
      >
        <BotAvatar color={bot.color} size={26} still />
        <span className="flex min-w-0 flex-1 flex-col gap-0.5">
          <span className="flex items-center gap-2 text-sm">
            <span className="truncate font-semibold">{bot.name}</span>
            {crew && <span className="truncate text-muted text-xs">{words.inCrew(crew.name)}</span>}
            <time
              className="ml-auto shrink-0 text-muted text-xs"
              dateTime={new Date(hit.item.createdAt).toISOString()}
            >
              {when(hit.item.createdAt)}
            </time>
          </span>
          <span className="line-clamp-2 text-ink-soft text-sm">
            <span className="font-medium text-ink">{author}: </span>
            {snippetParts(hit.snippet).map((part, index) =>
              part.found ? (
                // biome-ignore lint/suspicious/noArrayIndexKey: the parts never move
                <mark key={index} className="rounded-sm bg-warn/25 px-0.5 text-ink">
                  {part.text}
                </mark>
              ) : (
                // biome-ignore lint/suspicious/noArrayIndexKey: the parts never move
                <span key={index}>{part.text}</span>
              ),
            )}
          </span>
        </span>
      </button>
    </li>
  );
}
