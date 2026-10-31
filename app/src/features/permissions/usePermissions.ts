// Everything the owner let each agent do (spec 15.1), read when shown and
// again, gathered over 200 ms, whenever a grant, a rule or a site changes
// or a request in a chat is answered: an answer may have saved something.

import { useEffect, useState } from "react";
import type { Permissions } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";

const EMPTY: Permissions = { desktop: [], sites: [], rules: [], crewAccess: [] };
const GATHER_MS = 200;

export function usePermissions(): Permissions | null {
  const api = useApi();
  const [all, setAll] = useState<Permissions | null>(null);
  useEffect(() => {
    let alive = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const read = () =>
      api.call("permissions.list").then(
        (listed) => alive && setAll(listed),
        // An older daemon has no such list: nothing to show.
        () => alive && setAll((current) => current ?? EMPTY),
      );
    const later = () => {
      clearTimeout(timer);
      timer = setTimeout(read, GATHER_MS);
    };
    void read();
    const stop = api.subscribe((event) => {
      if (
        event.name === "bot.desktop" ||
        event.name === "bot.rules" ||
        event.name === "bot.sites"
      ) {
        later();
      } else if (event.name === "chat.item") {
        const body = event.params.item.body;
        if (body.kind === "approval" && body.status !== "pending") {
          later();
        }
      }
    });
    return () => {
      alive = false;
      clearTimeout(timer);
      stop();
    };
  }, [api]);
  return all;
}
