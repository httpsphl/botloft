import { useCallback, useEffect, useState } from "react";
import type { Build } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

/** The Maker's builds (spec 36.4), read when shown and after an undo. */
export function useBuilds() {
  const api = useApi();
  const [builds, setBuilds] = useState<Build[]>([]);
  const reload = useCallback(async () => {
    try {
      setBuilds(await api.call("builds.list"));
    } catch {
      // Without them the line just has no buttons.
    }
  }, [api]);
  useEffect(() => {
    void reload();
  }, [reload]);
  return { builds, reload };
}
