// Opening, saving and showing a bot's file (spec 15.1), here or from a
// server. From a server the file comes in pieces through the connection
// (42.7), and nothing of it is in a folder of this computer to show.

import { useMemo } from "react";
import type { BotFile } from "../../lib/protocol.gen";
import { botFile, openFrom, saveFrom } from "../../lib/transfer";
import { useServer } from "../../shell/server";
import { useApi, useHost } from "../../store/context";

type Named = Pick<BotFile, "path" | "name">;

export interface FileActions {
  open(botId: string, file: Named): Promise<void>;
  save(botId: string, file: Named): Promise<void>;
  /** Shows the file in its folder; null when the files are on another computer. */
  reveal: ((path: string) => Promise<void>) | null;
}

export function useFileActions(): FileActions {
  const host = useHost();
  const api = useApi();
  const remote = useServer() !== null;
  return useMemo(() => {
    if (remote) {
      return {
        open: (botId, file) => openFrom(host, file.name, botFile(api, botId, file.path)),
        save: async (botId, file) => {
          await saveFrom(host, file.name, botFile(api, botId, file.path));
        },
        reveal: null,
      };
    }
    return {
      open: (_botId, file) => host.openFile(file.path),
      save: async (_botId, file) => {
        await host.saveFileAs(file.path);
      },
      reveal: (path) => host.revealFile(path),
    };
  }, [host, api, remote]);
}

/** Whether folders open in this computer's file manager: not from a server. */
export function useFoldersHere(): boolean {
  return useServer() === null;
}
