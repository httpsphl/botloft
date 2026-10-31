// "Today" in the island's Home (spec 31.4): what the crews did since local
// midnight, counted by the daemon without an agent's turn and read as the
// home reads it (`useToday`).

import type { CSSProperties, ReactNode } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { DayActivity } from "../lib/protocol.gen";

function Count({ value, label }: { value: ReactNode; label: ReactNode }) {
  return (
    <span className="flex flex-col items-center text-center">
      <b className="text-[17px]">{value}</b>
      <span className="island-muted text-xs">{label}</span>
    </span>
  );
}

export function Today({ today }: { today: DayActivity }) {
  const t = useT();
  const words = t.island.today;
  const { added, removed } = today.lines;
  return (
    <div className="island-card flex items-center gap-4 px-4 py-3 text-[13px]">
      <div className="shrink-0">
        <div className="font-semibold">{words.title}</div>
        <div className="island-muted text-xs">{words.since}</div>
      </div>
      <div className="grid flex-1 grid-cols-4 gap-2">
        <Count
          value={today.tasksDone}
          label={
            <>
              {words.tasks(today.tasksDone)}
              {today.tasksFailed > 0 && (
                <span className="block text-[var(--island-danger)]">
                  {t.home.today.failed(today.tasksFailed)}
                </span>
              )}
            </>
          }
        />
        <Count value={today.requestsAnswered} label={words.requests(today.requestsAnswered)} />
        <Count
          value={
            <span className="font-mono text-[15px]">
              <span className="text-[var(--island-ok)]">+{added}</span>
              {removed > 0 && <span className="text-[var(--island-danger)]"> −{removed}</span>}
            </span>
          }
          label={words.lines(today.filesEdited)}
        />
        <Count value={today.botMessages} label={words.messages(today.botMessages)} />
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
