// One file in the files panel (spec 15.1): a row with its thumbnail or
// tile, its name and what it is ("Image · PNG"), or, among the pictures, a
// card that is mostly the picture.

import { useT } from "../../i18n";
import { fileSize, fromNow } from "../../lib/format";
import type { BotFile } from "../../lib/protocol.gen";
import { Badge } from "../../ui/Badge";
import { FileThumb } from "./FileThumb";
import { extensionOf, fileKind } from "./kinds";

/** What the file is, in words: "Image · PNG", "Spreadsheet · CSV". */
export function useKindLabel() {
  const t = useT().files;
  return (file: BotFile) => {
    const kind = fileKind(file.mediaType, file.name);
    return [t.kinds[kind], extensionOf(file.name)].filter(Boolean).join(" · ");
  };
}

export function FileRow({
  botId,
  file,
  fresh,
  onOpen,
}: {
  botId: string;
  file: BotFile;
  fresh: boolean;
  onOpen(path: string): void;
}) {
  const t = useT().files;
  const kind = fileKind(file.mediaType, file.name);
  const label = useKindLabel()(file);
  return (
    <li>
      <button
        type="button"
        onClick={() => onOpen(file.path)}
        title={file.path}
        className="flex w-full min-w-0 items-center gap-3 rounded-xl px-2 py-1.5 text-left transition-colors hover:bg-bubble"
      >
        <FileThumb botId={botId} file={file} kind={kind} size="row" />
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2">
            <span className="truncate font-medium text-sm">{file.name}</span>
            {fresh && <Badge tone="work">{t.newTag}</Badge>}
          </span>
          <span className="flex min-w-0 items-center gap-1.5 text-muted text-xs">
            <span className="file-kind shrink-0 font-medium" data-kind={kind}>
              {label}
            </span>
            <span aria-hidden>·</span>
            <span className="shrink-0">{fileSize(file.size)}</span>
            <span aria-hidden>·</span>
            <span className="truncate">
              {fromNow(file.modifiedAt)}
              {file.folder ? ` · ${file.folder}` : ""}
            </span>
          </span>
        </span>
      </button>
    </li>
  );
}

/** A picture among the pictures: big enough to see what it is. */
export function FileCard({
  botId,
  file,
  fresh,
  onOpen,
}: {
  botId: string;
  file: BotFile;
  fresh: boolean;
  onOpen(path: string): void;
}) {
  const t = useT().files;
  const kind = fileKind(file.mediaType, file.name);
  return (
    <li className="min-w-0">
      <button
        type="button"
        onClick={() => onOpen(file.path)}
        title={file.path}
        className="group flex w-full min-w-0 flex-col gap-1.5 rounded-xl p-1 text-left transition-colors hover:bg-bubble"
      >
        <span className="relative block">
          <FileThumb botId={botId} file={file} kind={kind} size="card" />
          {fresh && (
            <span className="absolute top-1.5 left-1.5">
              <Badge tone="work">{t.newTag}</Badge>
            </span>
          )}
        </span>
        <span className="min-w-0 px-0.5">
          <span className="block truncate font-medium text-xs">{file.name}</span>
          <span className="block truncate text-[11px] text-muted">
            {fileSize(file.size)} · {fromNow(file.modifiedAt)}
          </span>
        </span>
      </button>
    </li>
  );
}
