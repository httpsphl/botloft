// One file of a bot, shown in the panel: images, PDFs, markdown and plain
// text right there; anything else only offers to open it. A file is read
// from the daemon once per look, up to 20 MB.

import { ArrowLeft, ExternalLink, FolderOpen, LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { decodeBytes } from "../../lib/base64";
import { fileSize } from "../../lib/format";
import { type Bot, type BotFile, RpcErrorCode } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { Markdown } from "../chat/Markdown";
import { previewKind } from "./kinds";

/** Characters of text shown; a longer file shows its start. */
const TEXT_MAX = 200_000;

type Shown =
  | { kind: "loading" }
  | { kind: "failed"; gone: boolean; tooBig: boolean; detail: string }
  | { kind: "none" }
  | { kind: "image"; src: string; alt: string }
  | { kind: "pdf"; src: string }
  | { kind: "markdown" | "text"; text: string; cut: boolean };

function useShown(bot: Bot, file: BotFile): Shown {
  const api = useApi();
  const [shown, setShown] = useState<Shown>({ kind: "loading" });

  // biome-ignore lint/correctness/useExhaustiveDependencies: a file is read again when it changes
  useEffect(() => {
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
  }, [api, bot.id, file.path, file.modifiedAt, file.mediaType]);

  return shown;
}

export function FilePreview({ bot, file, onBack }: { bot: Bot; file: BotFile; onBack(): void }) {
  const t = useT().files;
  const host = useHost();
  const shown = useShown(bot, file);
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-1.5 border-line border-b px-2 py-2">
        <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
          {t.back}
        </Button>
        <span className="min-w-0 flex-1" />
        <Button
          size="sm"
          icon={FolderOpen}
          onClick={() => attempt(t.failed.reveal, () => host.revealFile(file.path))}
        >
          {t.reveal}
        </Button>
        <Button
          size="sm"
          variant="primary"
          icon={ExternalLink}
          onClick={() => attempt(t.failed.open, () => host.openFile(file.path))}
        >
          {t.open}
        </Button>
      </div>
      <div className="border-line border-b px-4 py-3">
        <h3 className="break-words font-semibold text-sm" data-selectable>
          {file.name}
        </h3>
        <p className="mt-0.5 text-muted text-xs">{fileSize(file.size)}</p>
      </div>
      <div className="flex min-h-0 flex-1 flex-col overflow-auto">
        <Body shown={shown} />
      </div>
    </div>
  );
}

function Body({ shown }: { shown: Shown }) {
  const t = useT().files.preview;
  switch (shown.kind) {
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
        <div className="grid min-h-0 flex-1 place-items-center bg-sunken p-3">
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
