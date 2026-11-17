// The requests waiting for the owner in every crew (spec 15.1), as
// `cards.list` gives them, with the phone's rules (28.5), the questions as
// cards, and the agents' requests for a key (32.5), which only open the chat.
// Read when the page opens and when the connection comes back, and again
// when a request, a question, a request for a key or a bot's state changes,
// the changes of 200 ms read once. The home answers questions from the
// store; the island (spec 31.5) shows these cards.

import { useCallback, useEffect, useRef, useState } from "react";
import type { ApprovalCard, KeyCard, QuestionCard } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const SETTLE_MS = 200;

export function useRequests(): {
  /** Oldest first; null until the first read. */
  requests: ApprovalCard[] | null;
  /** The open questions, oldest first; null until the first read. */
  questions: QuestionCard[] | null;
  /** The open requests for a key, oldest first; null until the first read. */
  keys: KeyCard[] | null;
  /** Takes an answered request off at once, before the next read. */
  drop(approvalId: string): void;
  /** Reads again soon, as after an answer that may not have gone through. */
  refresh(): void;
} {
  const api = useApi();
  const [requests, setRequests] = useState<ApprovalCard[] | null>(null);
  const [questions, setQuestions] = useState<QuestionCard[] | null>(null);
  const [keys, setKeys] = useState<KeyCard[] | null>(null);
  const soon = useRef(() => {});

  useEffect(() => {
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = () =>
      api.call("cards.list").then(
        (cards) => {
          if (alive) {
            setRequests(cards.approvals);
            setQuestions(cards.questions);
            // A daemon from before the requests for a key has no `keys`.
            setKeys(cards.keys ?? []);
          }
        },
        // An older daemon has no cards: nothing shows as waiting.
        () => {
          if (alive) {
            setRequests((current) => current ?? []);
            setQuestions((current) => current ?? []);
            setKeys((current) => current ?? []);
          }
        },
      );
    soon.current = () => {
      clearTimeout(timer);
      timer = setTimeout(read, SETTLE_MS);
    };
    const stop = api.subscribe((event) => {
      const kind = event.name === "chat.item" ? event.params.item.body.kind : null;
      if (
        kind === "approval" ||
        kind === "question" ||
        kind === "key_request" ||
        event.name === "question.changed" ||
        event.name === "bot.state"
      ) {
        soon.current();
      }
    });
    // What changed while the daemon was away, as across a restart.
    const stopConnection = api.onConnection((state) => {
      if (state.kind === "open") {
        soon.current();
      }
    });
    void read();
    return () => {
      alive = false;
      clearTimeout(timer);
      stop();
      stopConnection();
    };
  }, [api]);

  const drop = useCallback(
    (approvalId: string) =>
      setRequests((current) => current?.filter((card) => card.approvalId !== approvalId) ?? null),
    [],
  );
  const refresh = useCallback(() => soon.current(), []);
  return { requests, questions, keys, drop, refresh };
}
