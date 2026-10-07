// Restoring a copy (spec 14.2): the owner picks the file, then `RestoreFile`
// asks for its passphrase and swaps it in on a restart.

import { FolderOpen } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { RestoreFile } from "./RestoreFile";

export function BackupRestore() {
  const b = useT().account.backup;
  const host = useHost();
  const [file, setFile] = useState<string | null>(null);

  const pick = async () => {
    const picked = await host.pickFile(b.pickTitle, b.kind, ["botloft"]);
    if (picked) {
      setFile(picked);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <Button icon={FolderOpen} onClick={pick}>
          {b.pick}
        </Button>
        {file && (
          <span className="min-w-0 truncate font-mono text-muted text-xs" title={file}>
            {file}
          </span>
        )}
      </div>
      {file && <RestoreFile key={file} file={file} onCancel={() => setFile(null)} />}
    </div>
  );
}
