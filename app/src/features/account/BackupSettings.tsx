// Settings, "Backup" (spec 14.2, 27): save everything to one file locked with
// a passphrase, restore such a file, and keep light copies in the cloud. The
// daemon makes the copy; the owner picks where it goes with the system's Save
// dialog.

import { Download } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { BackupRestore } from "./BackupRestore";
import { backupError } from "./backupError";
import { CloudBackup } from "./CloudBackup";
import { PassphraseFields, usePassphrase } from "./PassphraseFields";
import { Section } from "./settingsParts";

type Outcome = { saved: string } | { notSaved: true } | { failed: string } | null;

export function BackupSettings({ goToAccount }: { goToAccount(): void }) {
  const b = useT().account.backup;
  return (
    <>
      <CloudBackup goToAccount={goToAccount} />
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
  const passphrase = usePassphrase();
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>(null);

  const save = async () => {
    setBusy(true);
    setOutcome(null);
    try {
      const exported = await api.call("backup.export", { passphrase: passphrase.passphrase });
      const kept = await host.saveFileAs(exported.path);
      setOutcome(kept ? { saved: fileSize(exported.size) } : { notSaved: true });
      if (kept) {
        passphrase.clear();
      }
    } catch (failure) {
      setOutcome({ failed: backupError(failure, b.reasons) });
    }
    setBusy(false);
  };

  return (
    <div className="flex flex-col gap-3">
      <PassphraseFields value={passphrase} />
      <Button
        icon={Download}
        className="self-start"
        disabled={!passphrase.ready || busy}
        onClick={save}
      >
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
