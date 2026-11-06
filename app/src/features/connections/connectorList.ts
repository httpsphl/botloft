// The owner's claude.ai tools (spec 34.4, 34.7): the list of the account's
// connectors, read once and again on demand, and the page that connects
// more. Signed in once on claude.ai, they need no sign-in here.

import { useCallback, useEffect, useState } from "react";
import type { ConnectorList } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

export const CONNECTORS_PAGE = "https://claude.ai/customize/connectors";

/** How old a list may be before the Tools page looks again on its own. */
const STALE_MS = 60_000;

/** Whether a list may be out of date: something in it is not connected, or
 * it has none, and the last look was a while ago (spec 34.3). */
function mayHaveChanged(list: ConnectorList): boolean {
  const unsettled =
    list.connectors.length === 0 || list.connectors.some((tool) => tool.state !== "connected");
  return unsettled && Date.now() - list.checkedAt > STALE_MS;
}

/** The account's connectors, looked for once and again on demand. With
 * `fresh`, it looks again by itself when the kept list may be out of date:
 * the owner may just have signed in to a tool on claude.ai. */
export function useConnectorList({ fresh = false }: { fresh?: boolean } = {}) {
  const api = useApi();
  const [list, setList] = useState<ConnectorList | null>(null);
  const [looking, setLooking] = useState(false);
  const look = useCallback(
    async (refresh: boolean) => {
      setLooking(true);
      try {
        const found = await api.call("connectors.list", { refresh });
        setList(found);
        return found;
      } catch {
        setList((current) => current ?? { connectors: [], checkedAt: 0 });
        return null;
      } finally {
        setLooking(false);
      }
    },
    [api],
  );
  useEffect(() => {
    void look(false).then((kept) => {
      if (fresh && kept && mayHaveChanged(kept)) {
        void look(true);
      }
    });
  }, [look, fresh]);
  return { list, looking, refresh: () => look(true) };
}
