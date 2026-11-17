// One file of the bot's as a card in its chat (spec 15.1): a picture that
// opens big, a sound or a video that plays right there, or the tile of its
// kind; then its name, kind and size, and buttons to open it and save a
// copy anywhere. Shared files (`share_file`) and what the bot made in a
// turn use the same card.

import { Download, ExternalLink } from "lucide-react";
import { useContext, useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { BotFile } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { ImageViewer } from "../files/ImageViewer";
import { fileKind, kindIcon } from "../files/kinds";
import { MediaPlayer } from "../files/MediaPlayer";
import { mediaKind } from "../files/media";
import { ShowFile } from "../files/showFile";
import { useFileImage } from "./images";

/** "PDF" from report.pdf; nothing for a name without one. */
function extension(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toUpperCase() : "";
}

function Picture({ botId, file }: { botId: string; file: BotFile }) {
  const t = useT().files.picture;
  const source = useFileImage(botId, file);
  const [big, setBig] = useState(false);
  if (source.kind !== "ready") {
    return null;
  }
  return (
    <>
      <button
        type="button"
        aria-label={t.enlarge(file.name)}
        title={t.enlarge(file.name)}
        onClick={() => setBig(true)}
        className="block w-full cursor-zoom-in bg-sunken"
      >
        <img src={source.url} alt={file.name} className="block max-h-56 w-full object-contain" />
      </button>
      {big && <ImageViewer src={source.url} name={file.name} onClose={() => setBig(false)} />}
    </>
  );
}

export function FileCard({ botId, file }: { botId: string; file: BotFile }) {
  const t = useT().files;
  const host = useHost();
  const showFile = useContext(ShowFile);
  const kind = fileKind(file.mediaType, file.name);
  const plays = mediaKind(file);
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
      {plays === "video" && <MediaPlayer botId={botId} file={file} kind="video" />}
      {!plays && <Picture botId={botId} file={file} />}
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
      {plays === "audio" && (
        <div className="border-line border-t">
          <MediaPlayer botId={botId} file={file} kind="audio" />
        </div>
      )}
    </div>
  );
}
