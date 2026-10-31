// "Today" in the island's Home (spec 31.4): what the crews did since local
// midnight, counted by the daemon without an agent's turn.

import { type CSSProperties, useEffect, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { DayActivity } from "../lib/protocol.gen";

/** The start of today on this computer, in Unix ms. */
export function localMidnight(now = Date.now()): number {
  const day = new Date(now);
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

/** Today's counts, read again whenever `version` changes. */
export function useToday(api: BotloftApi, version: number): DayActivity | null {
  const [today, setToday] = useState<DayActivity | null>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `version` asks to read again
  useEffect(() => {
    let alive = true;
    api.call("activity.today", { since: localMidnight() }).then(
      (read) => alive && setToday(read),
      () => {},
    );
    return () => {
      alive = false;
    };
  }, [api, version]);
  return today;
}

function Count({ value, label }: { value: string | number; label: string }) {
  return (
    <span className="flex flex-col items-center">
      <b className="text-[17px]">{value}</b>
      <span className="island-muted text-xs">{label}</span>
    </span>
  );
}

export function Today({ today }: { today: DayActivity }) {
  const t = useT();
  const words = t.island.today;
  return (
    <div className="island-card flex items-center gap-4 px-4 py-3 text-[13px]">
      <div className="shrink-0">
        <div className="font-semibold">{words.title}</div>
        <div className="island-muted text-xs">{words.since}</div>
      </div>
      <div className="grid flex-1 grid-cols-3 gap-2">
        <Count value={today.tasksDone} label={words.tasks(today.tasksDone)} />
        <Count value={today.requestsAnswered} label={words.requests(today.requestsAnswered)} />
        <span className="flex flex-col items-center">
          <b className="font-mono text-[15px] text-[var(--island-ok)]">+{today.lines.added}</b>
          <span className="island-muted text-xs">{words.lines(today.filesEdited)}</span>
        </span>
      </div>
      {today.busiest && (
        <div className="flex shrink-0 items-center gap-2 border-[var(--island-line)] border-l pl-4">
          <span className="island-glow" style={{ "--glow": today.busiest.color } as CSSProperties}>
            <BotAvatar color={today.busiest.color} size={26} mood="awake" still />
          </span>
          <div>
            <div className="font-medium">{today.busiest.name}</div>
            <div className="island-muted text-xs">{words.busiest}</div>
          </div>
        </div>
      )}
    </div>
  );
}
