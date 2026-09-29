// Files sent with a message: images as thumbnails that open larger, the
// rest as cards with name, type and size (spec 15.3).

import { File, FileImage, FileText, FolderOpen, LoaderCircle } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { Attachment } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { useImage } from "./images";

/** The folder an attachment was saved in, inside the bot's workspace. */
export function attachmentFolder(workspace: string, attachment: Attachment): string {
  const parts = attachment.path.split("/").slice(0, -1);
  return [workspace, ...parts].join("\\");
}

/** `file` is the word for a file of no known type. */
function kindLabel(mediaType: string, file: string): string {
  const [kind, sub] = mediaType.split("/");
  if (!sub || mediaType === "application/octet-stream") {
    return file;
  }
  return (sub.split(/[.+-]/).at(-1) ?? kind ?? "file").toUpperCase();
}

function FileCard({
  attachment,
  workspace,
  note,
}: {
  attachment: Attachment;
  workspace: string | null;
  note?: string;
}) {
  const host = useHost();
  const t = useT();
  const Icon = attachment.mediaType.startsWith("image/")
    ? FileImage
    : attachment.mediaType.startsWith("text/") || attachment.mediaType.endsWith("pdf")
      ? FileText
      : File;
  return (
    <div className="flex w-64 max-w-full items-center gap-2.5 rounded-md border border-line bg-panel py-2 pr-1.5 pl-2.5">
      <Icon aria-hidden size={22} className="shrink-0 text-muted" />
      <div className="min-w-0 flex-1">
        <p className="truncate font-medium text-sm" title={attachment.name}>
          {attachment.name}
        </p>
        <p className="truncate text-muted text-xs">
          {note ??
            `${kindLabel(attachment.mediaType, t.chat.attachments.file)} · ${fileSize(attachment.size)}`}
        </p>
      </div>
      {workspace && (
        <button
          type="button"
          aria-label={t.chat.attachments.showInFolder(attachment.name)}
          title={t.chat.attachments.showInFolderHint}
          onClick={() =>
            attempt(t.chat.attachments.openFolderFailed, () =>
              host.openPath(attachmentFolder(workspace, attachment)),
            )
          }
          className="grid h-7 w-7 shrink-0 place-items-center rounded-[3px] text-muted hover:bg-sunken hover:text-ink"
        >
          <FolderOpen aria-hidden size={14} />
        </button>
      )}
    </div>
  );
}

function Thumbnail({
  attachment,
  workspace,
}: {
  attachment: Attachment;
  workspace: string | null;
}) {
  const t = useT();
  const source = useImage(attachment);
  const [open, setOpen] = useState(false);
  if (source.kind === "none") {
    return <FileCard attachment={attachment} workspace={workspace} />;
  }
  if (source.kind === "missing") {
    return (
      <FileCard attachment={attachment} workspace={workspace} note={t.chat.attachments.missing} />
    );
  }
  if (source.kind === "loading") {
    return (
      <div className="grid h-32 w-44 place-items-center rounded-md border border-line bg-sunken">
        <LoaderCircle
          aria-label={t.chat.attachments.loading(attachment.name)}
          className="animate-spin text-muted"
        />
      </div>
    );
  }
  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="overflow-hidden rounded-md border border-line bg-sunken"
        title={attachment.name}
      >
        <img
          src={source.url}
          alt={attachment.name}
          className="block max-h-56 max-w-72 object-contain"
        />
      </button>
      {open && (
        <Dialog title={attachment.name} width="lg" onClose={() => setOpen(false)}>
          <img src={source.url} alt={attachment.name} className="mx-auto max-h-[70vh]" />
        </Dialog>
      )}
    </>
  );
}

export function AttachmentList({
  attachments,
  workspace,
  align = "start",
}: {
  attachments: Attachment[];
  /** The bot's folder, to show a file in it; `null` when unknown. */
  workspace: string | null;
  align?: "start" | "end";
}) {
  const t = useT();
  if (attachments.length === 0) {
    return null;
  }
  return (
    <ul
      aria-label={t.chat.attachments.label}
      className={`flex flex-wrap gap-2 ${align === "end" ? "justify-end" : ""}`}
    >
      {attachments.map((attachment) => (
        <li key={attachment.id} className="max-w-full">
          <Thumbnail attachment={attachment} workspace={workspace} />
        </li>
      ))}
    </ul>
  );
}
