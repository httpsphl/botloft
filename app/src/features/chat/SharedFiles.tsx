// Files the bot shared with `share_file` (spec 10, 15.3): one card each,
// with the file's tile or picture, its name, kind and size, and buttons to
// preview it, open it and save a copy anywhere.

import { Download, ExternalLink } from "lucide-react";
import { useContext } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { BotFile, ChatItem } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { useArrival } from "../../ui/motion";
import { attempt } from "../../ui/toast";
import { fileKind, kindIcon } from "../files/kinds";
import { ShowFile } from "../files/showFile";
import { useFileImage } from "./images";
import { sharedFiles } from "./shared";

/** "PDF" from report.pdf; nothing for a name without one. */
function extension(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toUpperCase() : "";
}

function Picture({ botId, file }: { botId: string; file: BotFile }) {
  const source = useFileImage(botId, file);
  if (source.kind !== "ready") {
    return null;
  }
  return (
    <img
      src={source.url}
      alt={file.name}
      className="block max-h-56 w-full bg-sunken object-contain"
    />
  );
}

function FileCard({ botId, file }: { botId: string; file: BotFile }) {
  const t = useT().files;
  const host = useHost();
  const showFile = useContext(ShowFile);
  const kind = fileKind(file.mediaType, file.name);
  const Icon = kindIcon(kind);
  const what = [extension(file.name), fileSize(file.size)].filter(Boolean).join(" · ");
  const about = (
    <>
      <span aria-hidden className="file-tile" data-kind={kind}>
        <Icon size={18} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate font-medium text-sm" title={file.path}>
          {file.name}
        </span>
        <span className="block truncate text-muted text-xs">{what}</span>
      </span>
    </>
  );
  return (
    <div className="@container flex w-96 max-w-full flex-col overflow-hidden rounded-xl border border-line bg-panel shadow-sm">
      <Picture botId={botId} file={file} />
      <div className="flex items-center gap-1 p-1.5">
        {/* Where there is a files panel, the file itself opens its preview. */}
        {showFile ? (
          <button
            type="button"
            aria-label={`${t.previewOne}: ${file.name}`}
            title={t.previewOne}
            onClick={() => showFile(file.path, file)}
            className="flex min-w-0 flex-1 items-center gap-2.5 rounded-lg p-1 text-left transition-colors hover:bg-sunken"
          >
            {about}
          </button>
        ) : (
          <div className="flex min-w-0 flex-1 items-center gap-2.5 p-1">{about}</div>
        )}
        <Button
          variant="ghost"
          size="sm"
          icon={ExternalLink}
          label={t.open}
          onClick={() => attempt(t.failed.open, () => host.openFile(file.path))}
        />
        {/* A narrow card keeps the name and drops the words on the button. */}
        <Button
          size="sm"
          icon={Download}
          aria-label={t.save}
          title={t.saveOne(file.name)}
          onClick={() => attempt(t.failed.save, () => host.saveFileAs(file.path))}
        >
          <span className="hidden @[22rem]:inline">{t.save}</span>
        </Button>
      </div>
    </div>
  );
}

export function SharedFiles({ item, botId, bot }: { item: ChatItem; botId: string; bot: string }) {
  const t = useT().files;
  const arrival = useArrival(item.createdAt);
  if (item.body.kind !== "tool") {
    return null;
  }
  const files = sharedFiles(item.body);
  if (files.length === 0) {
    return null;
  }
  return (
    <ul aria-label={t.shared(bot)} className={`flex flex-wrap gap-2 ${arrival}`}>
      {files.map((file) => (
        <li key={file.path} className="max-w-full">
          <FileCard botId={botId} file={file} />
        </li>
      ))}
    </ul>
  );
}
