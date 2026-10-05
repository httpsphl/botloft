// Settings, "Backup" (spec 14.2): save everything to one file locked with
// a passphrase, and restore such a file. The daemon makes the copy; the
// owner picks where it goes with the system's Save dialog.

import { Download } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { fileSize } from "../../lib/format";
import { RpcError } from "../../lib/rpc";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { TextField } from "../../ui/Field";
import { BackupRestore } from "./BackupRestore";
import { Section } from "./settingsParts";

/** A failure as the owner reads it: the daemon's reason, worded, or the error. */
export function backupError(failure: unknown, reasons: Record<string, string>): string {
  const reason = failure instanceof RpcError ? failure.reason : undefined;
  return (reason && reasons[reason]) || errorText(failure);
}

type Outcome = { saved: string } | { notSaved: true } | { failed: string } | null;

export function BackupSettings() {
  const b = useT().account.backup;
  return (
    <>
      <Section title={b.exportTitle}>
        <p className="text-ink-soft text-sm leading-relaxed">{b.exportIntro}</p>
        <ExportCopy />
      </Section>
      <Section title={b.importTitle}>
        <p className="text-ink-soft text-sm leading-relaxed">{b.importIntro}</p>
        <BackupRestore />
      </Section>
    </>
  );
}

function ExportCopy() {
  const b = useT().account.backup;
  const api = useApi();
  const host = useHost();
  const [passphrase, setPassphrase] = useState("");
  const [repeat, setRepeat] = useState("");
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>(null);
  const mismatch = repeat.length > 0 && repeat !== passphrase;
  const ready = [...passphrase].length >= 8 && repeat === passphrase;

  const save = async () => {
    setBusy(true);
    setOutcome(null);
    try {
      const exported = await api.call("backup.export", { passphrase });
      const kept = await host.saveFileAs(exported.path);
      setOutcome(kept ? { saved: fileSize(exported.size) } : { notSaved: true });
      if (kept) {
        setPassphrase("");
        setRepeat("");
      }
    } catch (failure) {
      setOutcome({ failed: backupError(failure, b.reasons) });
    }
    setBusy(false);
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <TextField
          type="password"
          autoComplete="new-password"
          label={b.passphrase}
          value={passphrase}
          onChange={(event) => setPassphrase(event.target.value)}
        />
        <TextField
          type="password"
          autoComplete="new-password"
          label={b.repeat}
          value={repeat}
          onChange={(event) => setRepeat(event.target.value)}
        />
      </div>
      <p className={`text-xs ${mismatch ? "text-danger" : "text-muted"}`}>
        {mismatch ? b.mismatch : b.passphraseHint}
      </p>
      <Button icon={Download} className="self-start" disabled={!ready || busy} onClick={save}>
        {busy ? b.exporting : b.export}
      </Button>
      {outcome && "saved" in outcome && (
        <p role="status" className="text-ok text-sm">
          {b.saved(outcome.saved)}
        </p>
      )}
      {outcome && "notSaved" in outcome && (
        <p role="status" className="text-muted text-sm">
          {b.notSaved}
        </p>
      )}
      {outcome && "failed" in outcome && (
        <Callout tone="danger" title={b.exportFailed}>
          {outcome.failed}
        </Callout>
      )}
    </div>
  );
}
