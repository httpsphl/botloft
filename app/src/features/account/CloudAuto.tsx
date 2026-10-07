// Copies sent by themselves (spec 27.10): every day or week, and only when
// something changed. The passphrase goes to the computer's credential store.

import { useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { AutoBackupEvery } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Select } from "../../ui/Select";
import { backupError } from "./backupError";
import { PassphraseFields, usePassphrase } from "./PassphraseFields";
import { useAutoBackup } from "./useAutoBackup";

export function CloudAuto() {
  const t = useT().account.cloud;
  const a = t.auto;
  const api = useApi();
  const { status, setStatus } = useAutoBackup();
  const passphrase = usePassphrase();
  const [every, setEvery] = useState<AutoBackupEvery>("daily");
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  if (!status) {
    return null;
  }
  const options: { value: AutoBackupEvery; label: string }[] = [
    { value: "daily", label: a.daily },
    { value: "weekly", label: a.weekly },
  ];

  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    setProblem(null);
    try {
      await work();
    } catch (failure) {
      setProblem(backupError(failure, t.reasons));
    }
    setBusy(false);
  };

  const turnOn = () =>
    run(async () => {
      setStatus(await api.call("autobackup.enable", { passphrase: passphrase.passphrase, every }));
      passphrase.clear();
    });
  const turnOff = () => run(async () => setStatus(await api.call("autobackup.disable")));
  const change = (next: AutoBackupEvery) =>
    run(async () => setStatus(await api.call("autobackup.set_every", { every: next })));

  return (
    <div className="flex flex-col gap-3 border-line border-t pt-3">
      <h4 className="font-medium text-sm">{a.title}</h4>
      {!status.available ? (
        <p className="text-muted text-sm">{a.unavailable}</p>
      ) : status.enabled ? (
        <>
          <p role="status" className="text-ok text-sm">
            {status.every === "weekly" ? a.onWeekly : a.onDaily}
          </p>
          <p className="text-ink-soft text-sm">
            {status.lastOkAt !== undefined ? a.lastCopy(when(status.lastOkAt)) : a.noneYet}
            {status.nextAt !== undefined && ` ${a.next(when(status.nextAt))}`}
          </p>
          {status.lastError && (
            <Callout tone="danger" title={a.lastFailed}>
              {t.reasons[status.lastError] ?? t.reasons.cloud_error}
            </Callout>
          )}
          <div className="flex items-end gap-3">
            <Select label={a.every} value={status.every} options={options} onChange={change} />
            <Button variant="ghost" disabled={busy} onClick={turnOff}>
              {a.turnOff}
            </Button>
          </div>
        </>
      ) : (
        <>
          <p className="text-ink-soft text-sm leading-relaxed">{a.intro}</p>
          <PassphraseFields value={passphrase} />
          <p className="text-muted text-xs">{a.tip}</p>
          <div className="flex items-end gap-3">
            <Select<AutoBackupEvery>
              label={a.every}
              value={every}
              options={options}
              onChange={setEvery}
            />
            <Button disabled={!passphrase.ready || busy} onClick={turnOn}>
              {busy ? a.turningOn : a.turnOn}
            </Button>
          </div>
        </>
      )}
      {problem && (
        <Callout tone="danger" title={a.failed}>
          {problem}
        </Callout>
      )}
    </div>
  );
}
