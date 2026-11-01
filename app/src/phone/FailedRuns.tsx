// A routine that failed, among what waits on the phone (spec 28.12): which
// agent's routine, and the way to its conversation. It goes once the owner
// reads that chat, here or on the computer.

import { CircleAlert } from "lucide-react";
import { useT } from "../i18n";
import type { ChatLine, RunFailure } from "../lib/protocol.gen";
import { BotDot } from "./parts";

/** The bot's failed routine the owner has not read past yet. */
export function failureOf(line: ChatLine, seen: Record<string, number>): RunFailure | null {
  const read = Math.max(seen[line.botId] ?? 0, line.readAt ?? 0);
  return line.failed && line.failed.at > read ? line.failed : null;
}

export function FailedRuns({
  chats,
  seen,
  open,
}: {
  chats: ChatLine[];
  seen: Record<string, number>;
  open(botId: string): void;
}) {
  const t = useT().phone.failed;
  const failed = chats
    .map((line) => ({ line, failure: failureOf(line, seen) }))
    .filter((entry): entry is { line: ChatLine; failure: RunFailure } => entry.failure !== null)
    .sort((a, b) => a.failure.at - b.failure.at);
  return (
    <>
      {failed.map(({ line, failure }) => (
        <section
          key={line.botId}
          aria-label={t.label(failure.routine, line.name)}
          className="flex flex-col gap-3 rounded-2xl border border-danger/45 bg-panel p-4"
        >
          <div className="flex items-center gap-3">
            <BotDot name={line.name} color={line.color} state={line.state} size={34} />
            <p className="min-w-0 flex-1 font-medium">
              <CircleAlert aria-hidden size={15} className="mr-1.5 inline text-danger" />
              {t.label(failure.routine, line.name)}
            </p>
          </div>
          <button
            type="button"
            onClick={() => open(line.botId)}
            className="h-12 rounded-xl border border-line-strong font-medium text-base active:scale-[0.99]"
          >
            {t.open}
          </button>
        </section>
      ))}
    </>
  );
}
