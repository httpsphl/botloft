// What the Settings screen knows of the phones (spec 28.5): their status,
// kept current by the daemon's notifications, and the time left on a code.

import { useCallback, useEffect, useState } from "react";
import type { MobileStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export function useMobile() {
  const api = useApi();
  const [status, setStatus] = useState<MobileStatus | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.call("mobile.status"));
    } catch {
      // The screen keeps what it had.
    }
  }, [api]);

  useEffect(() => {
    void refresh();
    return api.subscribe((event) => {
      if (event.name === "mobile.changed") {
        setStatus(event.params);
      }
    });
  }, [api, refresh]);

  return { status, refresh };
}

/** Seconds left until `expiresAt`, counted down each second; 0 when it is past. */
export function useSecondsLeft(expiresAt: number | undefined): number {
  const [seconds, setSeconds] = useState(() => secondsUntil(expiresAt));
  useEffect(() => {
    setSeconds(secondsUntil(expiresAt));
    const timer = setInterval(() => setSeconds(secondsUntil(expiresAt)), 1000);
    return () => clearInterval(timer);
  }, [expiresAt]);
  return seconds;
}

function secondsUntil(at: number | undefined): number {
  return at === undefined ? 0 : Math.max(0, Math.ceil((at - Date.now()) / 1000));
}

/** "4:32". */
export function clock(seconds: number): string {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}
