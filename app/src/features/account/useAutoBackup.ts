// What the account block knows of the automatic backup (spec 27.10): its
// status, kept current by the daemon's `autobackup.changed`.

import { useCallback, useEffect, useState } from "react";
import type { AutoBackupStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export function useAutoBackup() {
  const api = useApi();
  const [status, setStatus] = useState<AutoBackupStatus | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.call("autobackup.status"));
    } catch {
      // The screen keeps what it had.
    }
  }, [api]);

  useEffect(() => {
    void refresh();
    return api.subscribe((event) => {
      if (event.name === "autobackup.changed") {
        setStatus(event.params);
      }
    });
  }, [api, refresh]);

  return { status, setStatus };
}
