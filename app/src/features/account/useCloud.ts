// What the Settings screen knows of the account (spec 27.6): its status,
// kept current by the daemon's notifications, and how a transfer is going.

import { useCallback, useEffect, useState } from "react";
import type { CloudDirection, CloudProgress, CloudStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export function useCloud() {
  const api = useApi();
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [expired, setExpired] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.call("cloud.status"));
    } catch {
      // The screen keeps what it had.
    }
  }, [api]);

  useEffect(() => {
    void refresh();
    return api.subscribe((event) => {
      if (event.name === "cloud.signed_in") {
        setExpired(false);
        void refresh();
      } else if (event.name === "cloud.signin_expired") {
        setExpired(true);
        void refresh();
      }
    });
  }, [api, refresh]);

  return { status, refresh, expired, forgetExpired: () => setExpired(false) };
}

/** How much of a copy has gone `direction`, from 0 to 1, or null when nothing moves. */
export function useCloudProgress(direction: CloudDirection, active: boolean): number | null {
  const api = useApi();
  const [progress, setProgress] = useState<CloudProgress | null>(null);

  useEffect(() => {
    if (!active) {
      setProgress(null);
      return;
    }
    return api.subscribe((event) => {
      if (event.name === "cloud.progress" && event.params.direction === direction) {
        setProgress(event.params);
      }
    });
  }, [api, direction, active]);

  return active && progress && progress.total > 0
    ? Math.min(1, progress.sent / progress.total)
    : null;
}
