// What the island shows (spec 31.5): the requests, questions and requests
// for a key waiting, oldest first, read as the home reads them
// (`useRequests`), and the agents at work with their crews, read again
// 200 ms after the daemon says something that may change them.

import { useCallback, useEffect, useRef, useState } from "react";
import { useRequests } from "../features/home/useRequests";
import type { BotloftApi } from "../lib/api";
import type { ApprovalCard, Bot, Crew, KeyCard, QuestionCard } from "../lib/protocol.gen";

export type Waiting =
  | { kind: "approval"; id: string; card: ApprovalCard }
  | { kind: "question"; id: string; card: QuestionCard }
  | { kind: "key"; id: string; card: KeyCard };

/** How long the island waits for more news before reading again. */
const SETTLE_MS = 200;

export function useWaiting(api: BotloftApi): {
  waiting: Waiting[];
  /** The first read of what waits is in: what waited then is not news. */
  loaded: boolean;
  working: Bot[];
  bots: Bot[];
  /** Crew names by id, for the agents' cards. */
  crews: Map<string, string>;
  crewList: Crew[];
  /** Goes up each time the island reads the agents again. */
  version: number;
  refresh: () => void;
} {
  const cards = useRequests();
  const [bots, setBots] = useState<Bot[]>([]);
  const [crewList, setCrewList] = useState<Crew[]>([]);
  const [version, setVersion] = useState(0);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const load = useCallback(async () => {
    try {
      const [botList, crewsRead] = await Promise.all([
        api.call("bots.list", {}),
        api.call("crews.list"),
      ]);
      setBots(botList.filter((bot) => bot.archivedAt === null));
      setCrewList(crewsRead);
      setVersion((before) => before + 1);
    } catch {
      // Not connected yet: the next connection reads again.
    }
  }, [api]);

  const soon = useCallback(() => {
    clearTimeout(timer.current);
    timer.current = setTimeout(load, SETTLE_MS);
  }, [load]);

  const { refresh: refreshCards } = cards;
  const refresh = useCallback(() => {
    refreshCards();
    soon();
  }, [refreshCards, soon]);

  useEffect(() => {
    load();
    const offEvents = api.subscribe((event) => {
      if (event.name === "bot.state" || event.name === "crew.changed") {
        soon();
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
  }, [api, load, soon]);

  const waiting: Waiting[] = [
    ...(cards.requests ?? []).map((card) => ({
      kind: "approval" as const,
      id: card.approvalId,
      card,
    })),
    ...(cards.questions ?? []).map((card) => ({
      kind: "question" as const,
      id: card.questionId,
      card,
    })),
    ...(cards.keys ?? []).map((card) => ({ kind: "key" as const, id: card.requestId, card })),
  ].sort((a, b) => a.card.createdAt - b.card.createdAt);
  const working = bots.filter((bot) => bot.state === "busy");
  const crews = new Map(crewList.map((crew) => [crew.id, crew.name]));
  return {
    waiting,
    loaded: cards.requests !== null && cards.questions !== null && cards.keys !== null,
    working,
    bots,
    crews,
    crewList,
    version,
    refresh,
  };
}
