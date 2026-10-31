// "Today" on the home (spec 15.1): what the crews did since midnight, as
// the daemon counts it (`activity.today`, 31.4), without a turn of any
// agent. Read when the home opens and again when a turn ends, the turns of
// a second read once.

import { type ReactNode, useEffect, useState } from "react";
import { useT } from "../../i18n";
import { duration } from "../../lib/format";
import type { DayActivity } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { EditCounts } from "../chat/EditDiff";

const SETTLE_MS = 1000;

function midnight(): number {
  const day = new Date();
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

function useToday(): DayActivity | null {
  const api = useApi();
  const [today, setToday] = useState<DayActivity | null>(null);
  useEffect(() => {
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = () =>
      api.call("activity.today", { since: midnight() }).then(
        (day) => alive && setToday(day),
        // An older daemon does not count the day: the section stays out.
        () => {},
      );
    const stop = api.subscribe((event) => {
      if (event.name === "chat.item" && event.params.item.body.kind === "turn") {
        clearTimeout(timer);
        timer = setTimeout(read, SETTLE_MS);
      }
    });
    void read();
    return () => {
      alive = false;
      clearTimeout(timer);
      stop();
    };
  }, [api]);
  return today;
}

/** The day's numbers, or a line that nothing happened yet; nothing before the first read. */
export function Today() {
  const t = useT().home.today;
  const day = useToday();
  if (!day) {
    return null;
  }
  const quiet =
    day.tasksDone + day.tasksFailed + day.botMessages + day.requestsAnswered + day.filesEdited ===
      0 && !day.busiest;
  if (quiet) {
    return <p className="text-muted text-sm">{t.nothing}</p>;
  }
  return (
    <dl className="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3">
      <Tile label={t.tasks}>
        {day.tasksDone}
        {day.tasksFailed > 0 && (
          <span className="ml-1.5 font-normal text-danger text-xs">
            {t.failed(day.tasksFailed)}
          </span>
        )}
      </Tile>
      <Tile label={t.requests}>{day.requestsAnswered}</Tile>
      <Tile label={t.files}>
        {day.filesEdited}
        {day.filesEdited > 0 && (
          <span className="ml-1.5 font-normal text-xs">
            <EditCounts lines={day.lines} />
          </span>
        )}
      </Tile>
      <Tile label={t.messages}>{day.botMessages}</Tile>
      {day.busiest && (
        <Tile label={t.busiest}>
          <span className="truncate">{day.busiest.name}</span>
          <span className="ml-1.5 font-normal text-muted text-xs">
            {duration(day.busiest.workMs)}
          </span>
        </Tile>
      )}
    </dl>
  );
}

function Tile({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="min-w-0 rounded-xl border border-line bg-panel px-3.5 py-2.5">
      <dt className="text-muted text-xs">{label}</dt>
      <dd className="mt-0.5 flex items-baseline font-semibold text-lg tabular-nums">{children}</dd>
    </div>
  );
}
