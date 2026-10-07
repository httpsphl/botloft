// The account when this computer is in (spec 27.6): who, how much is used,
// the copies, and the way out.

import { useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { CloudStatus } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";
import { backupError } from "./backupError";
import { CloudAuto } from "./CloudAuto";
import { CloudCopies } from "./CloudCopies";
import { CloudUpload } from "./CloudUpload";

export function CloudSignedIn({
  status,
  refresh,
}: {
  status: CloudStatus;
  refresh(): Promise<void>;
}) {
  const t = useT().account.cloud;
  const api = useApi();
  const [version, setVersion] = useState(0);
  const [removing, setRemoving] = useState(false);
  const [asked, setAsked] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  const signOut = async () => {
    await api.call("cloud.signout").catch(() => null);
    await refresh();
  };

  const removeAccount = async () => {
    setProblem(null);
    try {
      await api.call("cloud.delete_account");
      setAsked(true);
    } catch (failure) {
      setProblem(backupError(failure, t.reasons));
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="truncate font-medium text-sm">{t.signedInAs(status.email ?? "")}</p>
          {status.used !== undefined && status.quota !== undefined && (
            <p className="text-muted text-xs">
              {t.usage(fileSize(status.used), fileSize(status.quota))}
            </p>
          )}
        </div>
        <Button variant="ghost" onClick={signOut}>
          {t.signOut}
        </Button>
      </div>

      <CloudUpload
        onSent={() => {
          setVersion((current) => current + 1);
          void refresh();
        }}
      />
      <CloudAuto />
      <CloudCopies version={version} onChanged={refresh} />

      <div className="flex flex-col gap-2 border-line border-t pt-3">
        {asked ? (
          <p role="status" className="text-ink-soft text-sm">
            {t.removeAccountSent}
          </p>
        ) : (
          <Button
            variant="ghost"
            className="self-start text-danger"
            onClick={() => setRemoving(true)}
          >
            {t.removeAccount}
          </Button>
        )}
        {problem && (
          <Callout tone="danger" title={t.failed}>
            {problem}
          </Callout>
        )}
      </div>
      {removing && (
        <Confirm
          title={t.removeAccountTitle}
          confirmLabel={t.removeAccountSend}
          onConfirm={removeAccount}
          onClose={() => setRemoving(false)}
        >
          {t.removeAccountText}
        </Confirm>
      )}
    </div>
  );
}
