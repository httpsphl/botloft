// What the island shows (spec 31.5): the requests and questions waiting,
// oldest first, and the agents at work. Read again, 200 ms after the
// daemon says something that may change them.

import { useCallback, useEffect, useRef, useState } from "react";
import type { BotloftApi } from "../lib/api";
import type { ApprovalCard, Bot, QuestionCard } from "../lib/protocol.gen";

export type Waiting =
  | { kind: "approval"; id: string; card: ApprovalCard }
  | { kind: "question"; id: string; card: QuestionCard };

/** How long the island waits for more news before reading again. */
const SETTLE_MS = 200;

export function useWaiting(api: BotloftApi): {
  waiting: Waiting[];
  working: Bot[];
  bots: Bot[];
  /** Crew names by id, for the agents' cards. */
  crews: Map<string, string>;
  /** Goes up each time the island reads again. */
  version: number;
  refresh: () => void;
} {
  const [waiting, setWaiting] = useState<Waiting[]>([]);
  const [bots, setBots] = useState<Bot[]>([]);
  const [crews, setCrews] = useState<Map<string, string>>(new Map());
  const [version, setVersion] = useState(0);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const load = useCallback(async () => {
    try {
      const [cards, botList, crewList] = await Promise.all([
        api.call("cards.list"),
        api.call("bots.list", {}),
        api.call("crews.list"),
      ]);
      const all: Waiting[] = [
        ...cards.approvals.map((card) => ({
          kind: "approval" as const,
          id: card.approvalId,
          card,
        })),
        ...cards.questions.map((card) => ({
          kind: "question" as const,
          id: card.questionId,
          card,
        })),
      ];
      setWaiting(all.sort((a, b) => a.card.createdAt - b.card.createdAt));
      setBots(botList.filter((bot) => bot.archivedAt === null));
      setCrews(new Map(crewList.map((crew) => [crew.id, crew.name])));
      setVersion((before) => before + 1);
    } catch {
      // Not connected yet: the next connection reads again.
    }
  }, [api]);

  const refresh = useCallback(() => {
    clearTimeout(timer.current);
    timer.current = setTimeout(load, SETTLE_MS);
  }, [load]);

  useEffect(() => {
    load();
    const offEvents = api.subscribe((event) => {
      const news =
        event.name === "bot.state" ||
        event.name === "question.changed" ||
        (event.name === "chat.item" &&
          (event.params.item.body.kind === "approval" ||
            event.params.item.body.kind === "question"));
      if (news) {
        refresh();
      }
    });
    const offConnection = api.onConnection((state) => {
      if (state.kind === "open") {
        load();
      }
    });
    return () => {
      offEvents();
      offConnection();
      clearTimeout(timer.current);
    };
  }, [api, load, refresh]);

  const working = bots.filter((bot) => bot.state === "busy");
  return { waiting, working, bots, crews, version, refresh };
}
