// A file the owner picks for Botloft to read, such as a backup or a history
// export: its path on this computer, or, with Botloft on a server, sent
// there first in pieces, and its path there (spec 42.7).

import { useCallback, useState } from "react";
import { sendPicked } from "../../lib/transfer";
import { useServer } from "../../shell/server";
import { useApi, useHost } from "../../store/context";

export function usePickForDaemon() {
  const host = useHost();
  const api = useApi();
  const remote = useServer() !== null;
  /** The name of the file on its way to the server. */
  const [sending, setSending] = useState<string | null>(null);

  /** The path Botloft reads it at; null when the owner cancels. */
  const pick = useCallback(
    async (title: string, kind: string, extensions: string[]): Promise<string | null> => {
      if (!remote) {
        return host.pickFile(title, kind, extensions);
      }
      const picked = await host.transfers.pick(title, kind, extensions);
      if (!picked) {
        return null;
      }
      setSending(picked.name);
      try {
        return await sendPicked(api, host, picked);
      } finally {
        setSending(null);
      }
    },
    [api, host, remote],
  );

  return { pick, sending };
}
