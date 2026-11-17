// The panel beside a bot's chat with the files it made (spec 15.1): newest
// first, the ones that showed up since the owner last looked marked new,
// pictures with their own thumbnail, and chips that keep one kind (the
// pictures as a grid). A click shows the file in the panel itself, which
// can take the whole room for it.

import { ArrowLeft, RefreshCw, X } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotFile } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { EmptyState } from "../../ui/EmptyState";
import { SidePanel } from "../../ui/SidePanel";
import { ComputerFooter } from "../terminal/ComputerDock";
import { FilePreview } from "./FilePreview";
import { FileCard, FileRow } from "./FileRow";
import { FILE_GROUPS, type FileGroup, fileKind, kindGroup } from "./kinds";
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
  // A file shown can take all the room the chat can spare, until it closes.
  const [wide, setWide] = useState(false);
  const back = () => {
    setWide(false);
    onPath(null);
  };

  return (
    <SidePanel
      label={t.panel(bot.name)}
      name="files"
      defaultWidth={400}
      expanded={wide && chosen !== undefined}
    >
      <header className="flex h-11 shrink-0 items-center justify-between pr-1.5 pl-4">
        <h2 className="font-semibold text-base">{t.heading}</h2>
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
        <FilePreview bot={bot} file={chosen} wide={wide} onWide={setWide} onBack={back} />
      ) : path !== null ? (
        <Missing onBack={back} />
      ) : (
        <List bot={bot} data={data} since={since} onOpen={onPath} />
      )}
      <ComputerFooter />
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
  const [only, setOnly] = useState<FileGroup | null>(null);
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
  const groupOf = (file: BotFile) => kindGroup(fileKind(file.mediaType, file.name));
  const counts = new Map<FileGroup, number>();
  for (const file of data.files) {
    counts.set(groupOf(file), (counts.get(groupOf(file)) ?? 0) + 1);
  }
  const present = FILE_GROUPS.filter((group) => counts.has(group));
  // A group that emptied out shows everything again.
  const shown = only !== null && counts.has(only) ? only : null;
  const files = shown === null ? data.files : data.files.filter((file) => groupOf(file) === shown);
  const chip = (group: FileGroup | null, label: string, count: number) => (
    <button
      key={group ?? "all"}
      type="button"
      aria-pressed={shown === group}
      onClick={() => setOnly(group)}
      className={`shrink-0 rounded-full px-2.5 py-1 font-medium text-xs transition-colors ${
        shown === group ? "bg-ink text-canvas" : "bg-bubble text-ink-soft hover:text-ink"
      }`}
    >
      {label} <span className="opacity-60">{count}</span>
    </button>
  );
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {present.length > 1 && (
        <fieldset
          aria-label={t.filter}
          className="m-0 flex flex-wrap gap-1.5 border-0 px-3 pt-0.5 pb-2"
        >
          {chip(null, t.groups.all, data.files.length)}
          {present.map((group) => chip(group, t.groups[group], counts.get(group) ?? 0))}
        </fieldset>
      )}
      {shown === "images" ? (
        <ul
          aria-label={t.list}
          className="grid min-h-0 flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(7.5rem,1fr))] gap-1 overflow-y-auto px-2 pb-2"
        >
          {files.map((file) => (
            <FileCard
              key={file.path}
              botId={bot.id}
              file={file}
              fresh={file.modifiedAt > since}
              onOpen={onOpen}
            />
          ))}
        </ul>
      ) : (
        <ul aria-label={t.list} className="min-h-0 flex-1 overflow-y-auto px-1.5 pb-1.5">
          {files.map((file) => (
            <FileRow
              key={file.path}
              botId={bot.id}
              file={file}
              fresh={file.modifiedAt > since}
              onOpen={onOpen}
            />
          ))}
        </ul>
      )}
    </div>
  );
}
