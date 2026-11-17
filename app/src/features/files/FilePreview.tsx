// One file of a bot, shown in the panel: images, PDFs, markdown and plain
// text right there, sounds and videos in a player; anything else only
// offers to open it. A file is read from the daemon once per look, up to
// 20 MB; a sound or video plays from its address, of any size.

import {
  ArrowLeft,
  ExternalLink,
  FolderOpen,
  LoaderCircle,
  Maximize2,
  Minimize2,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { decodeBytes } from "../../lib/base64";
import { fileSize, fromNow } from "../../lib/format";
import { type Bot, type BotFile, RpcErrorCode } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { Markdown } from "../chat/Markdown";
import { useKindLabel } from "./FileRow";
import { FileThumb } from "./FileThumb";
import { fileKind, previewKind } from "./kinds";
import { MediaPlayer } from "./MediaPlayer";
import { type MediaKind, mediaKind } from "./media";

/** Characters of text shown; a longer file shows its start. */
const TEXT_MAX = 200_000;

type Shown =
  | { kind: "loading" }
  | { kind: "failed"; gone: boolean; tooBig: boolean; detail: string }
  | { kind: "none" }
  | { kind: "media"; media: MediaKind }
  | { kind: "image"; src: string; alt: string }
  | { kind: "pdf"; src: string }
  | { kind: "markdown" | "text"; text: string; cut: boolean };

function useShown(bot: Bot, file: BotFile): Shown {
  const api = useApi();
  const [shown, setShown] = useState<Shown>({ kind: "loading" });

  // biome-ignore lint/correctness/useExhaustiveDependencies: a file is read again when it changes
  useEffect(() => {
    const media = mediaKind(file);
    if (media) {
      setShown({ kind: "media", media });
      return;
    }
    const kind = previewKind(file.mediaType);
    if (kind === "none") {
      setShown({ kind: "none" });
      return;
    }
    let alive = true;
    let url: string | null = null;
    setShown({ kind: "loading" });
    api
      .call("files.read", { botId: bot.id, path: file.path })
      .then((data) => {
        if (!alive) {
          return;
        }
        if (kind === "image") {
          setShown({ kind, src: `data:${data.mediaType};base64,${data.data}`, alt: file.name });
        } else if (kind === "pdf") {
          url = URL.createObjectURL(new Blob([decodeBytes(data.data)], { type: data.mediaType }));
          setShown({ kind, src: url });
        } else {
          const text = new TextDecoder().decode(decodeBytes(data.data));
          setShown({ kind, text: text.slice(0, TEXT_MAX), cut: text.length > TEXT_MAX });
        }
      })
      .catch((failure: unknown) => {
        if (alive) {
          const code = failure instanceof RpcError ? failure.code : 0;
          setShown({
            kind: "failed",
            gone: code === RpcErrorCode.notFound,
            tooBig: code === RpcErrorCode.validation,
            detail: errorText(failure),
          });
        }
      });
    return () => {
      alive = false;
      if (url) {
        URL.revokeObjectURL(url);
      }
    };
  }, [api, bot.id, file.path, file.modifiedAt, file.mediaType, file.name]);

  return shown;
}

export function FilePreview({
  bot,
  file,
  wide,
  onWide,
  onBack,
}: {
  bot: Bot;
  file: BotFile;
  /** The file takes all the room the chat can spare. */
  wide: boolean;
  onWide(wide: boolean): void;
  onBack(): void;
}) {
  const t = useT().files;
  const host = useHost();
  const shown = useShown(bot, file);
  const kind = fileKind(file.mediaType, file.name);
  const label = useKindLabel()(file);
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-1 px-2 pb-1">
        <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
          {t.back}
        </Button>
        <span className="min-w-0 flex-1" />
        <Button
          variant="ghost"
          size="sm"
          icon={wide ? Minimize2 : Maximize2}
          label={wide ? t.narrow : t.wide}
          aria-pressed={wide}
          onClick={() => onWide(!wide)}
        />
      </div>
      <div className="flex items-start gap-3 px-4 pb-3">
        <FileThumb botId={bot.id} file={file} kind={kind} size="row" />
        <div className="min-w-0 flex-1">
          <h3 className="break-words font-semibold text-sm leading-snug" data-selectable>
            {file.name}
          </h3>
          <p className="mt-0.5 text-muted text-xs">
            <span className="file-kind font-medium" data-kind={kind}>
              {label}
            </span>
            {` · ${fileSize(file.size)} · ${fromNow(file.modifiedAt)}`}
            {file.folder ? ` · ${file.folder}` : ""}
          </p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            <Button
              size="sm"
              variant="primary"
              icon={ExternalLink}
              onClick={() => attempt(t.failed.open, () => host.openFile(file.path))}
            >
              {t.open}
            </Button>
            <Button
              size="sm"
              icon={FolderOpen}
              onClick={() => attempt(t.failed.reveal, () => host.revealFile(file.path))}
            >
              {t.reveal}
            </Button>
          </div>
        </div>
      </div>
      <div className="mx-3 mb-3 flex min-h-0 flex-1 flex-col overflow-auto rounded-xl bg-bubble">
        <Body shown={shown} botId={bot.id} file={file} />
      </div>
    </div>
  );
}

function Body({ shown, botId, file }: { shown: Shown; botId: string; file: BotFile }) {
  const t = useT().files.preview;
  switch (shown.kind) {
    case "media":
      return (
        <div
          className={
            shown.media === "video" ? "grid min-h-0 flex-1 place-items-center bg-black" : "p-3"
          }
        >
          {shown.media === "audio" ? (
            <div className="rounded-xl border border-line bg-panel">
              <MediaPlayer botId={botId} file={file} kind="audio" />
            </div>
          ) : (
            <MediaPlayer botId={botId} file={file} kind="video" fill />
          )}
        </div>
      );
    case "loading":
      return (
        <p role="status" className="flex items-center gap-2 p-4 text-muted text-sm">
          <LoaderCircle aria-hidden size={14} className="animate-spin" />
          {t.loading}
        </p>
      );
    case "none":
      return <p className="p-4 text-muted text-sm">{t.none}</p>;
    case "failed":
      return (
        <div role="alert" className="p-4 text-sm">
          <p className="text-ink-soft">
            {shown.gone ? t.gone : shown.tooBig ? t.tooBig : t.failed}
          </p>
          {!shown.gone && !shown.tooBig && (
            <p className="mt-1 break-words font-mono text-muted text-xs">{shown.detail}</p>
          )}
        </div>
      );
    case "image":
      return (
        <div className="grid min-h-0 flex-1 place-items-center p-3">
          <img src={shown.src} alt={shown.alt} className="max-h-full max-w-full object-contain" />
        </div>
      );
    case "pdf":
      return <iframe title="PDF" src={shown.src} className="min-h-0 flex-1 border-0 bg-panel" />;
    case "markdown":
      return (
        <div className="p-4">
          <Markdown text={shown.text} />
          {shown.cut && <p className="mt-3 text-muted text-xs">{t.cut}</p>}
        </div>
      );
    case "text":
      return (
        <div className="p-4">
          <pre
            className="whitespace-pre-wrap break-words font-mono text-xs leading-relaxed"
            data-selectable
          >
            {shown.text}
          </pre>
          {shown.cut && <p className="mt-3 text-muted text-xs">{t.cut}</p>}
        </div>
      );
  }
}
