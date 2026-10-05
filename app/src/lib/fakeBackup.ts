// The fake daemon's backups (spec 14.2): what an export would hold, from
// the fake crews, at a made-up path. Nothing is sealed or written.

import type { FakeBotloft, Handlers } from "./fake";
import { RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type BackupMethods = Extract<keyof Handlers, `backup.${string}`>;

export function backupHandlers(fake: FakeBotloft): Pick<Handlers, BackupMethods> {
  return {
    "backup.export": ({ passphrase }) => {
      if ([...passphrase].length < 8) {
        throw new RpcError(
          RpcErrorCode.validation,
          "the passphrase must have at least 8 characters",
          "short_passphrase",
        );
      }
      const crews = [...fake.crews.values()];
      return {
        path: "C:\\Users\\owner\\AppData\\Local\\Botloft\\exports\\botloft-2026-10-05-1430.botloft",
        size: 1_234_567,
        manifest: {
          format: 1,
          createdAt: fake.now,
          version: "0.10.0",
          crews: crews.map((crew) => ({
            name: crew.name,
            bots: [...fake.bots.values()]
              .filter((bot) => bot.crewId === crew.id && bot.archivedAt === null)
              .map((bot) => bot.name),
            workFolder: crew.workFolderChosen ? crew.workFolder : null,
          })),
        },
      };
    },
  };
}
