import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { BotTemplate } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { notifyError } from "../../ui/toast";

/**
 * The roles of the Bot agency, read once when the screen opens. With
 * `enabled` false nothing is read (a dialog that only sometimes needs them).
 */
export function useCatalog(enabled = true): { roles: BotTemplate[] | null; failed: boolean } {
  const t = useT();
  const api = useApi();
  const [roles, setRoles] = useState<BotTemplate[] | null>(null);
  const [failed, setFailed] = useState(false);
  const failure = t.catalog.failed.load;

  useEffect(() => {
    if (!enabled) {
      return;
    }
    let live = true;
    api.call("catalog.list", {}).then(
      (list) => live && setRoles(list),
      (error) => {
        if (live) {
          setFailed(true);
          notifyError(failure, error);
        }
      },
    );
    return () => {
      live = false;
    };
  }, [api, failure, enabled]);

  return { roles, failed };
}
