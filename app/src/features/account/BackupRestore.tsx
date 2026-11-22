// Restoring a copy (spec 14.2): the owner picks the file, then `RestoreFile`
// asks for its passphrase and swaps it in on a restart. With Botloft on a
// server, the file goes there first (spec 42.7).

import { FolderOpen, LoaderCircle } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { usePickForDaemon } from "../files/pickForDaemon";
import { RestoreFile } from "./RestoreFile";

export function BackupRestore() {
  const t = useT();
  const b = t.account.backup;
  const { pick: pickFile, sending } = usePickForDaemon();
  const [file, setFile] = useState<string | null>(null);
  const [problem, setProblem] = useState<string | null>(null);

  const pick = async () => {
    setProblem(null);
    try {
      const picked = await pickFile(b.pickTitle, b.kind, ["botloft"]);
      if (picked) {
        setFile(picked);
      }
    } catch (failure) {
      setProblem(errorText(failure));
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <Button icon={FolderOpen} disabled={sending !== null} onClick={pick}>
          {b.pick}
        </Button>
        {sending ? (
          <span role="status" className="flex items-center gap-2 text-muted text-xs">
            <LoaderCircle aria-hidden size={14} className="animate-spin" />
            {t.server.sending(sending)}
          </span>
        ) : (
          file && (
            <span className="min-w-0 truncate font-mono text-muted text-xs" title={file}>
              {file}
            </span>
          )
        )}
      </div>
      {problem && (
        <Callout tone="danger" title={t.server.sendFailed}>
          {problem}
        </Callout>
      )}
      {file && <RestoreFile key={file} file={file} onCancel={() => setFile(null)} />}
    </div>
  );
}
