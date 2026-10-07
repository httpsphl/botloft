// The fake daemon's backups (spec 14.2): what an export would hold, from
// the fake crews, at a made-up path, and a backup that opens only with
// `FAKE_PASSPHRASE`. Nothing is sealed, written or restored.

import type { FakeBotloft, Handlers } from "./fake";
import { type BackupManifest, type BackupScope, RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

type BackupMethods = Extract<keyof Handlers, `backup.${string}`>;

/** The passphrase the fake's backups open with. */
export const FAKE_PASSPHRASE = "correct horse";

function refused(reason: string, message: string): RpcError {
  return new RpcError(RpcErrorCode.validation, message, reason);
}

function manifestOf(fake: FakeBotloft, scope: BackupScope = "full"): BackupManifest {
  return {
    format: 1,
    createdAt: fake.now,
    version: "0.11.0",
    scope,
    crews: [...fake.crews.values()].map((crew) => ({
      name: crew.name,
      bots: [...fake.bots.values()]
        .filter((bot) => bot.crewId === crew.id && bot.archivedAt === null)
        .map((bot) => bot.name),
      workFolder: crew.workFolderChosen ? crew.workFolder : null,
    })),
  };
}

export function backupHandlers(fake: FakeBotloft): Pick<Handlers, BackupMethods> {
  return {
    "backup.export": ({ passphrase, scope }) => {
      if ([...passphrase].length < 8) {
        throw refused("short_passphrase", "the passphrase must have at least 8 characters");
      }
      return {
        path: "C:\\Users\\owner\\AppData\\Local\\Botloft\\exports\\botloft-2026-10-05-1430.botloft",
        size: 1_234_567,
        manifest: manifestOf(fake, scope ?? "full"),
      };
    },
    "backup.stage": ({ path, passphrase }) => {
      if (passphrase !== FAKE_PASSPHRASE) {
        throw refused("wrong_passphrase", "the passphrase is wrong, or the file is damaged");
      }
      // A copy that came down from the cloud is a light one (spec 14.2).
      return manifestOf(fake, path.includes("cloud-download") ? "light" : "full");
    },
    "backup.confirm": () => null,
    "backup.cancel": () => null,
  };
}
