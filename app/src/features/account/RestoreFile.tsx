// Restoring a copy that is a file on this computer (spec 14.2): the owner
// types its passphrase, the daemon opens it and says what it holds, and only
// after the owner confirms does Botloft restart and swap it in. The file
// comes from the picker, or from the cloud (spec 27).

import { RotateCcw } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { BackupManifest } from "../../lib/protocol.gen";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { TextField } from "../../ui/Field";
import { backupError } from "./backupError";

export function RestoreFile({ file, onCancel }: { file: string; onCancel(): void }) {
  const b = useT().account.backup;
  const api = useApi();
  const host = useHost();
  const [passphrase, setPassphrase] = useState("");
  const [manifest, setManifest] = useState<BackupManifest | null>(null);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<{ title: string; text: string } | null>(null);

  const open = async () => {
    setBusy(true);
    setFailed(null);
    try {
      setManifest(await api.call("backup.stage", { path: file, passphrase }));
    } catch (failure) {
      setFailed({ title: b.openFailed, text: backupError(failure, b.reasons) });
    }
    setBusy(false);
  };

  const restore = async () => {
    setBusy(true);
    setFailed(null);
    try {
      await api.call("backup.confirm");
      await host.restartDaemon();
    } catch (failure) {
      setFailed({ title: b.restoreFailed, text: backupError(failure, b.reasons) });
      setBusy(false);
    }
  };

  const cancel = async () => {
    await api.call("backup.cancel").catch(() => null);
    onCancel();
  };

  const folders = (manifest?.crews ?? []).flatMap((crew) =>
    crew.workFolder ? [crew.workFolder] : [],
  );

  return (
    <div className="flex flex-col gap-3">
      {!manifest && (
        <div className="flex items-end gap-3">
          <div className="min-w-0 flex-1">
            <TextField
              type="password"
              autoComplete="current-password"
              label={b.copyPassphrase}
              value={passphrase}
              onChange={(event) => setPassphrase(event.target.value)}
            />
          </div>
          <Button disabled={busy || passphrase.length === 0} onClick={open}>
            {busy ? b.opening : b.open}
          </Button>
        </div>
      )}
      {manifest && (
        <div className="flex flex-col gap-2 rounded-lg border border-line bg-sunken px-3 py-2.5 text-sm">
          <p className="font-medium">{b.from(when(manifest.createdAt))}</p>
          <ul className="flex flex-col gap-0.5 text-ink-soft">
            {manifest.crews.map((crew) => (
              <li key={crew.name}>{b.crew(crew.name, crew.bots.length)}</li>
            ))}
          </ul>
          {manifest.scope === "light" && <p className="text-muted text-xs">{b.lightNote}</p>}
          {folders.length > 0 && (
            <div className="text-muted text-xs">
              <p>{b.workFolders}</p>
              <ul className="mt-1 font-mono" data-selectable>
                {folders.map((folder) => (
                  <li key={folder} className="break-all">
                    {folder}
                  </li>
                ))}
              </ul>
            </div>
          )}
          <p className="text-warn text-xs">{b.replaces}</p>
          <div className="mt-1 flex flex-wrap gap-2">
            <Button variant="danger" icon={RotateCcw} disabled={busy} onClick={restore}>
              {busy ? b.restoring : b.restore}
            </Button>
            <Button variant="ghost" disabled={busy} onClick={cancel}>
              {b.cancel}
            </Button>
          </div>
        </div>
      )}
      {failed && (
        <Callout tone="danger" title={failed.title}>
          {failed.text}
        </Callout>
      )}
    </div>
  );
}
