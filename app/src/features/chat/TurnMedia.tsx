// "Also made": the pictures, sounds and videos the bot made in a turn and
// did not share (turnFiles.ts), as the same cards a shared file gets, so
// the owner sees and plays them without digging through its commands.

import { useContext, useMemo } from "react";
import { useT } from "../../i18n";
import type { ChatItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { FileCard } from "./FileCard";
import { madeInTurn, sharedIn, TURN_MEDIA_MAX, TurnFiles } from "./turnFiles";

export function TurnMedia({
  items,
  turn,
  botId,
  bot,
}: {
  /** The run the turn ends. */
  items: ChatItem[];
  turn: ChatItem;
  botId: string;
  bot: string;
}) {
  const t = useT().files.made;
  const known = useContext(TurnFiles);
  const arrival = useArrival(turn.createdAt);
  const durationMs = turn.body.kind === "turn" ? turn.body.durationMs : 0;
  const made = useMemo(
    () =>
      known ? madeInTurn(known.files, { at: turn.createdAt, durationMs }, sharedIn(items)) : [],
    [known, turn.createdAt, durationMs, items],
  );
  if (!known || made.length === 0) {
    return null;
  }
  const more = made.length - TURN_MEDIA_MAX;
  return (
    <section aria-label={t.label(bot)} className={`flex flex-col gap-1.5 ${arrival}`}>
      <h3 className="px-1 font-normal text-muted text-xs">{t.heading}</h3>
      <ul className="flex flex-wrap gap-2">
        {made.slice(0, TURN_MEDIA_MAX).map((file) => (
          <li key={file.path} className="max-w-full">
            <FileCard botId={botId} file={file} />
          </li>
        ))}
      </ul>
      {more > 0 && (
        <button
          type="button"
          onClick={known.open}
          className="w-fit rounded px-1 text-muted text-xs underline-offset-2 transition-colors hover:text-ink hover:underline"
        >
          {t.more(more)}
        </button>
      )}
    </section>
  );
}
