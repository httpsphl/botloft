// A bot's screens, kept current (spec 22.4): read when the bot is opened,
// again when a write of one ends or the bot stops working, and merged with
// the drafts it streams while it writes one.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { errorText } from "../../lib/api";
import type { Bot, Screen, ScreenDraft } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";

export interface BotScreens {
  /** Newest first, with the drafts being written on top. */
  screens: Screen[];
  /** The path of a screen being written, if any. */
  writing: string | null;
  /** Bytes written so far of each screen being written, by lower-case path. */
  sizes: Record<string, number>;
  loading: boolean;
  error: string | null;
  refresh(): Promise<void>;
}

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();
const isHtml = (path: string) => /\.html?$/i.test(path);

/** The file name at the end of a path. */
export function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

export function useScreens(bot: Pick<Bot, "id" | "state">): BotScreens {
  const api = useApi();
  const open = useApp((state) => state.connection.kind === "open");
  const [list, setList] = useState<Screen[]>([]);
  const [drafts, setDrafts] = useState<ScreenDraft[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const latest = useRef(0);
  const botId = bot.id;

  const refresh = useCallback(async () => {
    latest.current += 1;
    const mine = latest.current;
    try {
      const found = await api.call("screens.list", { botId });
      if (mine === latest.current) {
        setList(found);
        setError(null);
      }
    } catch (failure) {
      if (mine === latest.current) {
        setError(errorText(failure));
      }
    } finally {
      if (mine === latest.current) {
        setLoading(false);
      }
    }
  }, [api, botId]);

  useEffect(() => {
    if (!open) {
      return;
    }
    setList([]);
    setDrafts([]);
    setLoading(true);
    void refresh();
  }, [open, refresh]);

  useEffect(
    () =>
      api.subscribe((event) => {
        if (event.name === "screen.draft" && event.params.botId === botId) {
          const draft = event.params;
          setDrafts((current) => {
            const others = current.filter((other) => !same(other.path, draft.path));
            return draft.done ? others : [...others, draft];
          });
          if (draft.done) {
            void refresh();
          }
        } else if (event.name === "chat.item" && event.params.item.botId === botId) {
          // An Edit of a screen changes it without a draft.
          const body = event.params.item.body;
          if (body.kind === "tool" && body.status === "done" && body.file && isHtml(body.file)) {
            void refresh();
          }
        }
      }),
    [api, botId, refresh],
  );

  const working = bot.state === "busy" || bot.state === "needs_approval";
  const wasWorking = useRef(working);
  useEffect(() => {
    if (open && wasWorking.current && !working) {
      void refresh();
    }
    wasWorking.current = working;
  }, [open, working, refresh]);

  const screens = useMemo(() => {
    const merged = list.map((screen) => {
      const draft = drafts.find((one) => same(one.path, screen.path));
      return draft ? { ...screen, url: draft.url, writing: true } : screen;
    });
    const fresh = drafts
      .filter((draft) => !list.some((screen) => same(screen.path, draft.path)))
      .map(
        (draft): Screen => ({
          path: draft.path,
          name: fileName(draft.path),
          folder: "",
          modifiedAt: Date.now(),
          url: draft.url,
          device: null,
          writing: true,
        }),
      );
    return [...fresh, ...merged];
  }, [list, drafts]);

  const sizes = useMemo(
    () => Object.fromEntries(drafts.map((draft) => [draft.path.toLowerCase(), draft.bytes])),
    [drafts],
  );

  return {
    screens,
    writing: drafts.at(-1)?.path ?? null,
    sizes,
    loading,
    error,
    refresh,
  };
}
