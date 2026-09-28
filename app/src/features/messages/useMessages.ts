// A page of a crew's or a bot's messages, kept current: new ones arrive by
// `message.created`, older ones load on demand, and a new connection
// reloads the newest page.

import { useCallback, useEffect, useState } from "react";
import { errorText } from "../../lib/api";
import type { BotId, CrewId, Message } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";

export type MessageFilter =
  | { crewId: CrewId; botId?: undefined }
  | { botId: BotId; crewId?: undefined };

const PAGE = 50;

interface Page {
  /** Oldest first. */
  messages: Message[];
  /** True when there is nothing older to load. */
  complete: boolean;
  loading: boolean;
  error: string | null;
}

/** Adds `message` unless the page already has it. */
function withMessage(page: Page, message: Message): Page {
  return page.messages.some((known) => known.id === message.id)
    ? page
    : { ...page, messages: [...page.messages, message] };
}

export function useMessages({ crewId, botId }: MessageFilter) {
  const api = useApi();
  const open = useApp((state) => state.connection.kind === "open");
  const [page, setPage] = useState<Page>({
    messages: [],
    complete: false,
    loading: true,
    error: null,
  });

  const params = useCallback(
    () => (crewId !== undefined ? { crewId } : botId !== undefined ? { botId } : {}),
    [crewId, botId],
  );

  useEffect(() => {
    if (!open) {
      return;
    }
    let alive = true;
    const fail = (error: unknown) =>
      alive && setPage((current) => ({ ...current, loading: false, error: errorText(error) }));
    setPage((current) => ({ ...current, loading: true, error: null }));
    api.call("messages.list", { ...params(), limit: PAGE }).then((newest) => {
      if (alive) {
        const messages = newest.reverse();
        setPage({ messages, complete: newest.length < PAGE, loading: false, error: null });
      }
    }, fail);
    const unsubscribe = api.subscribe((event) => {
      if (event.name !== "message.created") {
        return;
      }
      const message = event.params;
      const mine =
        crewId !== undefined
          ? message.crewId === crewId
          : message.toBotId === botId || message.fromBotId === botId;
      if (mine) {
        setPage((current) => withMessage(current, message));
      }
    });
    return () => {
      alive = false;
      unsubscribe();
    };
  }, [api, open, params, crewId, botId]);

  const loadOlder = useCallback(async () => {
    const oldest = page.messages[0];
    if (!oldest || page.complete) {
      return;
    }
    setPage((current) => ({ ...current, loading: true }));
    try {
      const older = await api.call("messages.list", {
        ...params(),
        before: oldest.id,
        limit: PAGE,
      });
      setPage((current) => ({
        messages: [...older.reverse(), ...current.messages],
        complete: older.length < PAGE,
        loading: false,
        error: null,
      }));
    } catch (error) {
      setPage((current) => ({ ...current, loading: false, error: errorText(error) }));
    }
  }, [api, params, page.messages, page.complete]);

  /** Adds a message a call returned, before its notification arrives. */
  const add = useCallback(
    (message: Message) => setPage((current) => withMessage(current, message)),
    [],
  );

  return { ...page, loadOlder, add };
}
