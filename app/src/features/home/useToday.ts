// What the crews did since midnight, as the daemon counts it
// (`activity.today`, spec 31.4), without a turn of any agent. Read when the
// page opens and again when a turn ends, the turns of a second read once.
// The home's "Today" (15.1) and the island's (31.4) show it.

import { useEffect, useState } from "react";
import type { DayActivity } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const SETTLE_MS = 1000;

/** The start of the day on this computer, in Unix ms: the time zone is the owner's. */
export function midnight(now = Date.now()): number {
  const day = new Date(now);
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

/** The day's numbers; null before the first read, and with an older daemon. */
export function useToday(): DayActivity | null {
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
