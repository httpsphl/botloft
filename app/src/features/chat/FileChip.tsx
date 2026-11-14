// A file waiting to go with the next message, in the composer (spec 9.5).

import { File as FileIcon, X } from "lucide-react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { PendingFile } from "./useFiles";

export function FileChip({ file, onRemove }: { file: PendingFile; onRemove(): void }) {
  const t = useT();
  return (
    <li className="relative flex h-14 items-center gap-2 rounded-xl border border-line bg-canvas pr-7 pl-1.5">
      {file.preview ? (
        <img src={file.preview} alt="" className="h-11 w-11 rounded-lg object-cover" />
      ) : (
        <FileIcon aria-hidden size={20} className="mx-1 text-muted" />
      )}
      <div className="min-w-0 max-w-40">
        <p className="truncate font-medium text-xs" title={file.name}>
          {file.name}
        </p>
        <p className="text-muted text-xs">{fileSize(file.size)}</p>
      </div>
      <button
        type="button"
        aria-label={t.chat.composer.remove(file.name)}
        onClick={onRemove}
        className="absolute top-1 right-1 grid h-5 w-5 place-items-center rounded-full text-muted hover:bg-sunken hover:text-ink"
      >
        <X aria-hidden size={12} />
      </button>
    </li>
  );
}
