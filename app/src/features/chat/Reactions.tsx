// The owner's reactions to what the bot wrote (spec 8.9): a button beside
// the reply button opens the emoji, and the one picked sits under the reply,
// saying whether the bot will see it with the next message or already did.
// A 👍 or 👎 on a reply that asks something goes at once, as the answer.

import { SmilePlus } from "lucide-react";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useT } from "../../i18n";
import { REACTIONS, type Reaction } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";
import { answers } from "./asks";

export interface Reactions {
  byItem: Record<string, Reaction>;
  set(itemId: string, emoji: string | null): void;
}

/** The open chat's reactions; absent where none can be put. */
export const ReactionsOf = createContext<Reactions | null>(null);

/** The bot's reactions, loaded on each connection and kept current. */
export function useReactions(botId: string): Reactions {
  const api = useApi();
  const t = useT().chat.react;
  const open = useApp((state) => state.connection.kind === "open");
  const [byItem, setByItem] = useState<Record<string, Reaction>>({});
  useEffect(() => {
    if (!open) return;
    let live = true;
    void api.call("reactions.list", { botId }).then(
      (list) => live && setByItem(Object.fromEntries(list.map((one) => [one.itemId, one]))),
      () => undefined,
    );
    const stop = api.subscribe((event) => {
      if (event.name !== "reaction.changed" || event.params.botId !== botId) return;
      const { itemId, reaction } = event.params;
      setByItem((before) => {
        const { [itemId]: _, ...rest } = before;
        return reaction ? { ...rest, [itemId]: reaction } : rest;
      });
    });
    return () => {
      live = false;
      stop();
    };
  }, [api, botId, open]);
  const set = useCallback(
    (itemId: string, emoji: string | null) =>
      attempt(t.failed, () => api.call("reactions.set", { botId, itemId, emoji })),
    [api, botId, t.failed],
  );
  return useMemo(() => ({ byItem, set }), [byItem, set]);
}

/**
 * Opens the emoji to react with. On a reply that `asked` something, 👍 and
 * 👎 answer it at once, and the emoji say so.
 */
export function ReactButton({
  itemId,
  bot,
  asked = false,
}: {
  itemId: string;
  bot: string;
  asked?: boolean;
}) {
  const t = useT().chat.react;
  const reactions = useContext(ReactionsOf);
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);
  if (!reactions) {
    return null;
  }
  const current = reactions.byItem[itemId]?.emoji ?? null;
  return (
    <div ref={root} className="relative">
      <button
        type="button"
        title={t.action}
        aria-label={t.to(bot)}
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="grid size-7 place-items-center rounded-full text-muted transition-colors hover:bg-sunken hover:text-ink"
      >
        <SmilePlus aria-hidden size={15} />
      </button>
      {open && (
        <div className="absolute right-0 bottom-full z-20 mb-1 flex animate-rise flex-col items-end gap-1">
          {asked && (
            <p className="whitespace-nowrap rounded-full border border-line bg-panel px-2.5 py-1 text-ink-soft text-xs shadow-lift">
              {t.answerHint}
            </p>
          )}
          <div
            role="menu"
            aria-label={t.to(bot)}
            className="flex gap-0.5 rounded-full border border-line bg-panel p-1 shadow-lift"
          >
            {REACTIONS.map((emoji) => {
              const says = answers(emoji, asked)
                ? emoji === "👍"
                  ? t.answerYes
                  : t.answerNo
                : undefined;
              return (
                <button
                  key={emoji}
                  type="button"
                  role="menuitemradio"
                  aria-checked={emoji === current}
                  aria-label={says && `${emoji} ${says}`}
                  title={says}
                  onClick={() => {
                    close();
                    reactions.set(itemId, emoji === current ? null : emoji);
                  }}
                  className={`grid size-8 place-items-center rounded-full text-lg transition-transform hover:scale-125 ${
                    emoji === current ? "bg-accent/15" : ""
                  }`}
                >
                  {emoji}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

/** Under the reply: the owner's reaction; a click takes it off. */
export function ReactionMark({
  itemId,
  bot,
  asked = false,
}: {
  itemId: string;
  bot: string;
  asked?: boolean;
}) {
  const t = useT().chat.react;
  const reactions = useContext(ReactionsOf);
  const reaction = reactions?.byItem[itemId];
  if (!reactions || !reaction) {
    return null;
  }
  const seen = reaction.sentIn !== null;
  const status = !seen
    ? t.waiting(bot)
    : answers(reaction.emoji, asked)
      ? t.answered(bot)
      : t.seen(bot);
  return (
    <div className="mt-1.5 flex animate-rise items-center gap-2">
      <button
        type="button"
        title={t.remove}
        aria-label={`${reaction.emoji} · ${t.remove}`}
        onClick={() => reactions.set(itemId, null)}
        className="flex h-7 items-center rounded-full border border-accent/40 bg-accent/10 px-2 text-base transition-colors hover:bg-accent/20"
      >
        {reaction.emoji}
      </button>
      <span className="text-muted text-xs">{status}</span>
    </div>
  );
}
