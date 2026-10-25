// The island folded (spec 31.2): how many wait on the left, the mascot of
// who matters most in the middle, how many work on the right. When an agent
// finishes, it widens for a moment to say so, the mascot hopping (31.4).

import { Check, LoaderCircle } from "lucide-react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { Cheer } from "./useCheer";

export const PILL = { width: 200, height: 36 };
export const CHEER_WIDTH = 460;

export function CheerPill({ cheer, onOpen }: { cheer: Cheer; onOpen: () => void }) {
  const t = useT();
  return (
    <button
      type="button"
      onClick={onOpen}
      className="flex h-9 w-full items-center gap-2.5 px-3.5 text-left text-[13px]"
    >
      <span className="island-hop inline-grid shrink-0">
        <BotAvatar color={cheer.bot.color} size={24} mood="awake" still />
      </span>
      <span className="min-w-0 flex-1 truncate">
        <b className="font-semibold">{t.island.finished(cheer.bot.name)}</b>
        {cheer.line && <span className="island-muted"> · {cheer.line}</span>}
      </span>
      <Check aria-hidden size={15} className="shrink-0 text-[var(--island-ok)]" />
    </button>
  );
}

export function Pill({
  waiting,
  working,
  color,
  onOpen,
}: {
  waiting: number;
  working: number;
  color: string;
  onOpen: () => void;
}) {
  const t = useT();
  const said = [waiting > 0 && t.island.waiting(waiting), working > 0 && t.island.working(working)]
    .filter(Boolean)
    .join(", ");
  return (
    <button
      type="button"
      onClick={onOpen}
      aria-label={said}
      title={said}
      className="flex h-9 w-full items-center justify-between px-3.5 text-[13px]"
    >
      <span className="flex w-10 items-center gap-1.5">
        {waiting > 0 && (
          <>
            <span className="size-2 rounded-full bg-[var(--island-warn)]" />
            <span className="font-medium text-[var(--island-warn)]">{waiting}</span>
          </>
        )}
      </span>
      <span className={waiting > 0 ? "inline-grid" : "island-breathe inline-grid"}>
        <BotAvatar color={color} size={24} mood={waiting > 0 ? "waiting" : "working"} still />
      </span>
      <span className="island-muted flex w-10 items-center justify-end gap-1">
        {working > 0 && (
          <>
            <LoaderCircle
              aria-hidden
              size={12}
              className="animate-spin text-[var(--island-work)]"
            />
            {working}
          </>
        )}
      </span>
    </button>
  );
}
