// One bot's chat, kept current (spec 8.3 and 15.1): the newest page on
// every connection, older pages on demand, items that are new or changed
// by `chat.item`, and the text being written by `chat.delta`.

import { useCallback, useEffect, useState } from "react";
import { errorText } from "../../lib/api";
import type { BotId, ChatItem, ChatItemId } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";

const PAGE = 50;

export interface ChatPage {
  /** Oldest first. */
  items: ChatItem[];
  /** Text of the reply being written, until the reply itself arrives. */
  draft: string;
  /** True when there is nothing older to load. */
  complete: boolean;
  loading: boolean;
  error: string | null;
}

const EMPTY: ChatPage = { items: [], draft: "", complete: false, loading: true, error: null };

/** Adds `item`, or replaces the copy the page has when `item` is newer. */
export function withItem(items: ChatItem[], item: ChatItem): ChatItem[] {
  const at = items.findIndex((known) => known.id === item.id);
  if (at < 0) {
    return [...items, item];
  }
  const known = items[at] as ChatItem;
  if (known.updatedAt > item.updatedAt) {
    return items;
  }
  return items.map((entry, index) => (index === at ? item : entry));
}

/** Items a page of history (newest first) and live updates agree on. */
function merge(history: ChatItem[], live: ChatItem[]): ChatItem[] {
  let items = [...history].reverse();
  for (const item of live) {
    items = withItem(items, item);
  }
  return items;
}

/** With `focus`, the chat loads from that item to the newest (spec 8.8). */
export function useChat(botId: BotId, focus: ChatItemId | null = null) {
  const api = useApi();
  const open = useApp((state) => state.connection.kind === "open");
  const [page, setPage] = useState<ChatPage>(EMPTY);

  useEffect(() => {
    if (!open) {
      return;
    }
    let alive = true;
    setPage({ ...EMPTY });
    // Subscribed first: what arrives while the page loads is newer.
    const unsubscribe = api.subscribe((event) => {
      if (event.name === "chat.item" && event.params.item.botId === botId) {
        const { item } = event.params;
        // A reply takes the place of its live text; a turn ends it.
        const ends = item.body.kind === "reply" || item.body.kind === "turn";
        setPage((current) => ({
          ...current,
          items: withItem(current.items, item),
          draft: ends ? "" : current.draft,
        }));
      } else if (event.name === "chat.delta" && event.params.botId === botId) {
        const { text } = event.params;
        setPage((current) => ({ ...current, draft: current.draft + text }));
      }
    });
    const newest = () => api.call("chat.history", { botId, limit: PAGE });
    // An item that is gone opens the chat as usual.
    const first = focus
      ? api.call("chat.history", { botId, until: focus }).catch(newest)
      : newest();
    first.then(
      (history) => {
        if (alive) {
          setPage((current) => ({
            ...current,
            items: merge(history, current.items),
            // From an item, older ones may still be there.
            complete: !focus && history.length < PAGE,
            loading: false,
          }));
        }
      },
      (error) => {
        if (alive) {
          setPage((current) => ({ ...current, loading: false, error: errorText(error) }));
        }
      },
    );
    return () => {
      alive = false;
      unsubscribe();
    };
  }, [api, open, botId, focus]);

  const oldest = page.items[0]?.id;
  const loadOlder = useCallback(async () => {
    if (!oldest) {
      return;
    }
    setPage((current) => ({ ...current, loading: true }));
    try {
      const older = await api.call("chat.history", { botId, before: oldest, limit: PAGE });
      setPage((current) => ({
        ...current,
        items: [...older.reverse(), ...current.items],
        complete: older.length < PAGE,
        loading: false,
        error: null,
      }));
    } catch (error) {
      setPage((current) => ({ ...current, loading: false, error: errorText(error) }));
    }
  }, [api, botId, oldest]);

  return { ...page, loadOlder };
}
