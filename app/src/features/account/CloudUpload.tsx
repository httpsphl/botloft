// Sending a light copy to the cloud (spec 27.6): the passphrase twice, then
// a bar while it goes up.

import { CloudUpload as UploadIcon } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { backupError } from "./backupError";
import { PassphraseFields, usePassphrase } from "./PassphraseFields";
import { ProgressBar } from "./ProgressBar";
import { useCloudProgress } from "./useCloud";

type Outcome = { sent: string } | { failed: string } | null;

export function CloudUpload({ onSent }: { onSent(): void }) {
  const t = useT().account.cloud;
  const api = useApi();
  const passphrase = usePassphrase();
  const [busy, setBusy] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>(null);
  const progress = useCloudProgress("upload", busy);

  const send = async () => {
    setBusy(true);
    setOutcome(null);
    try {
      const copy = await api.call("cloud.upload", { passphrase: passphrase.passphrase });
      setOutcome({ sent: fileSize(copy.size) });
      passphrase.clear();
      onSent();
    } catch (failure) {
      setOutcome({ failed: backupError(failure, t.reasons) });
    }
    setBusy(false);
  };

  return (
    <div className="flex flex-col gap-3">
      <h4 className="font-medium text-sm">{t.uploadTitle}</h4>
      <p className="text-ink-soft text-sm leading-relaxed">{t.uploadIntro}</p>
      <PassphraseFields value={passphrase} />
      <Button
        icon={UploadIcon}
        className="self-start"
        disabled={!passphrase.ready || busy}
        onClick={send}
      >
        {busy ? t.uploading : t.upload}
      </Button>
      {busy && progress !== null && <ProgressBar value={progress} label={t.uploading} />}
      {outcome && "sent" in outcome && (
        <p role="status" className="text-ok text-sm">
          {t.uploaded(outcome.sent)}
        </p>
      )}
      {outcome && "failed" in outcome && (
        <Callout tone="danger" title={t.uploadFailed}>
          {outcome.failed}
        </Callout>
      )}
    </div>
  );
}
