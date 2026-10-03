// The panel beside a bot's chat with the files it made (spec 15.1): newest
// first, the ones that showed up since the owner last looked marked new. A
// click shows the file in the panel itself.

import { ArrowLeft, RefreshCw, X } from "lucide-react";
import { useT } from "../../i18n";
import { fileSize, fromNow } from "../../lib/format";
import type { Bot, BotFile } from "../../lib/protocol.gen";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { EmptyState } from "../../ui/EmptyState";
import { SidePanel } from "../../ui/SidePanel";
import { FilePreview } from "./FilePreview";
import { fileKind, kindIcon } from "./kinds";
import type { BotFiles } from "./useBotFiles";

export function FilesPanel({
  bot,
  data,
  since,
  path,
  described,
  onPath,
  onClose,
}: {
  bot: Bot;
  data: BotFiles;
  /** Files modified after this (Unix ms) are marked new. */
  since: number;
  /** The file shown, or null for the list. */
  path: string | null;
  /** The file at `path` as the chat described it, if the list leaves it out. */
  described?: BotFile | null;
  onPath(path: string | null): void;
  onClose(): void;
}) {
  const t = useT().files;
  const chosen =
    data.files.find((file) => file.path === path) ??
    (described?.path === path ? described : undefined);

  return (
    <SidePanel label={t.panel(bot.name)} name="files" defaultWidth={416}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">{t.heading}</h2>
        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="sm"
            icon={RefreshCw}
            label={t.refresh}
            onClick={data.refresh}
          />
          <Button variant="ghost" size="sm" icon={X} label={t.close} onClick={onClose} />
        </div>
      </header>
      {chosen ? (
        <FilePreview bot={bot} file={chosen} onBack={() => onPath(null)} />
      ) : path !== null ? (
        <Missing onBack={() => onPath(null)} />
      ) : (
        <List bot={bot} data={data} since={since} onOpen={onPath} />
      )}
    </SidePanel>
  );
}

/** A file the chat pointed to that the folders no longer have. */
function Missing({ onBack }: { onBack(): void }) {
  const t = useT().files;
  return (
    <div className="flex flex-col items-start gap-3 p-4">
      <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
        {t.back}
      </Button>
      <p role="alert" className="text-ink-soft text-sm">
        {t.preview.gone}
      </p>
    </div>
  );
}

function List({
  bot,
  data,
  since,
  onOpen,
}: {
  bot: Bot;
  data: BotFiles;
  since: number;
  onOpen(path: string): void;
}) {
  const t = useT().files;
  if (data.error && data.files.length === 0) {
    return (
      <div className="p-3">
        <Callout tone="danger" title={t.loadFailed}>
          {data.error}
        </Callout>
      </div>
    );
  }
  if (data.files.length === 0 && !data.loading) {
    return <EmptyState color={bot.color} title={t.emptyTitle(bot.name)} body={t.emptyBody} />;
  }
  return (
    <ul aria-label={t.list} className="min-h-0 flex-1 overflow-y-auto p-1.5">
      {data.files.map((file) => (
        <FileRow key={file.path} file={file} fresh={file.modifiedAt > since} onOpen={onOpen} />
      ))}
    </ul>
  );
}

function FileRow({
  file,
  fresh,
  onOpen,
}: {
  file: BotFile;
  fresh: boolean;
  onOpen(path: string): void;
}) {
  const t = useT().files;
  const kind = fileKind(file.mediaType, file.name);
  const Icon = kindIcon(kind);
  return (
    <li>
      <button
        type="button"
        onClick={() => onOpen(file.path)}
        title={file.path}
        className="flex w-full min-w-0 items-center gap-3 rounded-lg px-2.5 py-2 text-left transition-colors hover:bg-sunken"
      >
        <span aria-hidden className="file-tile" data-kind={kind}>
          <Icon size={18} />
        </span>
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2">
            <span className="truncate font-medium text-sm">{file.name}</span>
            {fresh && <Badge tone="work">{t.newTag}</Badge>}
          </span>
          <span className="block truncate text-muted text-xs">
            {file.folder ? `${file.folder} · ` : ""}
            {fileSize(file.size)} · {fromNow(file.modifiedAt)}
          </span>
        </span>
      </button>
    </li>
  );
}
