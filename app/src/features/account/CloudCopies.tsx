// The copies kept in the cloud (spec 27.6): when each is from and how big,
// to bring back or delete.

import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import { fileSize, when } from "../../lib/format";
import type { CloudCopy } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";
import { backupError } from "./backupError";
import { ProgressBar } from "./ProgressBar";
import { RestoreFile } from "./RestoreFile";
import { useCloudProgress } from "./useCloud";

export function CloudCopies({
  version,
  onChanged,
}: {
  version: number;
  onChanged(): Promise<void>;
}) {
  const t = useT().account.cloud;
  const api = useApi();
  const [copies, setCopies] = useState<CloudCopy[] | null>(null);
  const [bringing, setBringing] = useState<string | null>(null);
  const [file, setFile] = useState<string | null>(null);
  const [removing, setRemoving] = useState<CloudCopy | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  const progress = useCloudProgress("download", bringing !== null);

  const load = useCallback(async () => {
    try {
      setCopies((await api.call("cloud.copies")).copies);
    } catch (failure) {
      setCopies([]);
      setProblem(backupError(failure, t.reasons));
    }
  }, [api, t.reasons]);

  // `version` changes when a copy was just sent.
  // biome-ignore lint/correctness/useExhaustiveDependencies: reload on a new copy
  useEffect(() => {
    void load();
  }, [load, version]);

  const bring = async (copy: CloudCopy) => {
    setBringing(copy.id);
    setProblem(null);
    try {
      setFile((await api.call("cloud.download", { id: copy.id })).path);
    } catch (failure) {
      setProblem(backupError(failure, t.reasons));
    }
    setBringing(null);
  };

  const remove = async (copy: CloudCopy) => {
    try {
      await api.call("cloud.delete", { id: copy.id });
    } catch (failure) {
      setProblem(backupError(failure, t.reasons));
    }
    await load();
    await onChanged();
  };

  if (!copies) {
    return null;
  }
  return (
    <div className="flex flex-col gap-2">
      <h4 className="font-medium text-sm">{t.copiesTitle}</h4>
      {copies.length === 0 ? (
        <p className="text-muted text-sm">{t.noCopies}</p>
      ) : (
        <ul className="flex flex-col divide-y divide-line rounded-lg border border-line">
          {copies.map((copy) => (
            <li key={copy.id} className="flex items-center gap-3 px-3 py-2">
              <span className="min-w-0 flex-1 truncate text-sm">
                {t.copyFrom(when(copy.created), fileSize(copy.size))}
              </span>
              <Button disabled={bringing !== null} onClick={() => bring(copy)}>
                {bringing === copy.id ? t.bringing : t.restore}
              </Button>
              <Button
                variant="ghost"
                disabled={bringing !== null}
                onClick={() => setRemoving(copy)}
              >
                {t.removeCopy}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {bringing !== null && progress !== null && (
        <ProgressBar value={progress} label={t.bringing} />
      )}
      {file && <RestoreFile key={file} file={file} onCancel={() => setFile(null)} />}
      {problem && (
        <Callout tone="danger" title={t.failed}>
          {problem}
        </Callout>
      )}
      {removing && (
        <Confirm
          title={t.removeCopyTitle}
          confirmLabel={t.removeCopy}
          onConfirm={() => remove(removing)}
          onClose={() => setRemoving(null)}
        >
          {t.removeCopyText}
        </Confirm>
      )}
    </div>
  );
}
