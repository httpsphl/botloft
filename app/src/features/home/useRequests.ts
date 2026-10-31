// The requests waiting for the owner in every crew (spec 15.1), as
// `cards.list` gives them, with the phone's rules (28.5). Read when the home
// opens and again when a request or a bot's state changes, the changes of
// 200 ms read once. The island (spec 31.5) reads them the same way.

import { useCallback, useEffect, useState } from "react";
import type { ApprovalCard } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const SETTLE_MS = 200;

export function useRequests(): {
  /** Oldest first; null until the first read. */
  requests: ApprovalCard[] | null;
  /** Takes an answered request off at once, before the next read. */
  drop(approvalId: string): void;
} {
  const api = useApi();
  const [requests, setRequests] = useState<ApprovalCard[] | null>(null);

  useEffect(() => {
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = () =>
      api.call("cards.list").then(
        (cards) => alive && setRequests(cards.approvals),
        // An older daemon has no cards: nothing shows as waiting.
        () => alive && setRequests((current) => current ?? []),
      );
    const soon = () => {
      clearTimeout(timer);
      timer = setTimeout(read, SETTLE_MS);
    };
    const stop = api.subscribe((event) => {
      if (
        (event.name === "chat.item" && event.params.item.body.kind === "approval") ||
        event.name === "bot.state"
      ) {
        soon();
      }
    });
    void read();
    return () => {
      alive = false;
      clearTimeout(timer);
      stop();
    };
  }, [api]);

  const drop = useCallback(
    (approvalId: string) =>
      setRequests((current) => current?.filter((card) => card.approvalId !== approvalId) ?? null),
    [],
  );
  return { requests, drop };
}
