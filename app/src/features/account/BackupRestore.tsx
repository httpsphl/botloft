// Restoring a copy (spec 14.2): the owner picks the file and types its
// passphrase; the daemon opens it and says what it holds; only after the
// owner confirms does Botloft restart and swap it in.

import { FolderOpen, RotateCcw } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { BackupManifest } from "../../lib/protocol.gen";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { TextField } from "../../ui/Field";
import { backupError } from "./BackupSettings";

export function BackupRestore() {
  const b = useT().account.backup;
  const api = useApi();
  const host = useHost();
  const [file, setFile] = useState<string | null>(null);
  const [passphrase, setPassphrase] = useState("");
  const [manifest, setManifest] = useState<BackupManifest | null>(null);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<{ title: string; text: string } | null>(null);

  const pick = async () => {
    const picked = await host.pickFile(b.pickTitle, b.kind, ["botloft"]);
    if (picked) {
      setFile(picked);
      setManifest(null);
      setFailed(null);
    }
  };

  const open = async () => {
    if (!file) {
      return;
    }
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
    setManifest(null);
    setFile(null);
    setPassphrase("");
  };

  const folders = (manifest?.crews ?? []).flatMap((crew) =>
    crew.workFolder ? [crew.workFolder] : [],
  );

  return (
    <div className="flex flex-col gap-3">
      {!manifest && (
        <>
          <div className="flex items-center gap-3">
            <Button icon={FolderOpen} onClick={pick} disabled={busy}>
              {b.pick}
            </Button>
            {file && (
              <span className="min-w-0 truncate font-mono text-muted text-xs" title={file}>
                {file}
              </span>
            )}
          </div>
          {file && (
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
        </>
      )}
      {manifest && (
        <div className="flex flex-col gap-2 rounded-lg border border-line bg-sunken px-3 py-2.5 text-sm">
          <p className="font-medium">{b.from(when(manifest.createdAt))}</p>
          <ul className="flex flex-col gap-0.5 text-ink-soft">
            {manifest.crews.map((crew) => (
              <li key={crew.name}>{b.crew(crew.name, crew.bots.length)}</li>
            ))}
          </ul>
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
