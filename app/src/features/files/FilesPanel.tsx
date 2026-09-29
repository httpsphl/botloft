// The panel beside a bot's chat with the files it made (spec 15.1): newest
// first, the ones that showed up since the owner last looked marked new. A
// click shows the file in the panel itself.

import { RefreshCw, X } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { fileSize, fromNow } from "../../lib/format";
import type { Bot, BotFile } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { BotAvatar } from "../bots/BotAvatar";
import { FilePreview } from "./FilePreview";
import { fileIcon } from "./kinds";
import type { BotFiles } from "./useBotFiles";

export function FilesPanel({
  bot,
  data,
  since,
  onClose,
}: {
  bot: Bot;
  data: BotFiles;
  /** Files modified after this (Unix ms) are marked new. */
  since: number;
  onClose(): void;
}) {
  const t = useT().files;
  const [path, setPath] = useState<string | null>(null);
  const chosen = data.files.find((file) => file.path === path);

  return (
    <aside
      aria-label={t.panel(bot.name)}
      className="flex w-[26rem] shrink-0 flex-col border-line border-l bg-panel"
    >
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
        <FilePreview bot={bot} file={chosen} onBack={() => setPath(null)} />
      ) : (
        <List bot={bot} data={data} since={since} onOpen={setPath} />
      )}
    </aside>
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
    return (
      <div className="flex flex-col items-center gap-3 px-6 py-14 text-center">
        <BotAvatar color={bot.color} size={44} mood="idle" />
        <p className="font-semibold text-sm">{t.emptyTitle(bot.name)}</p>
        <p className="text-muted text-sm">{t.emptyBody}</p>
      </div>
    );
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
  const Icon = fileIcon(file.mediaType, file.name);
  return (
    <li>
      <button
        type="button"
        onClick={() => onOpen(file.path)}
        title={file.path}
        className="flex w-full min-w-0 items-center gap-3 rounded-lg px-2.5 py-2 text-left transition-colors hover:bg-sunken"
      >
        <Icon aria-hidden size={18} className="shrink-0 text-muted" />
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2">
            <span className="truncate font-medium text-sm">{file.name}</span>
            {fresh && (
              <span className="shrink-0 rounded-full bg-work/12 px-1.5 py-px font-medium text-work text-xs">
                {t.newTag}
              </span>
            )}
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
