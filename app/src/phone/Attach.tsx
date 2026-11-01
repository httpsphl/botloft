// Photos and files the owner sends from the phone (spec 28.14): the two
// buttons by the box, what was picked and is about to go, and the names of
// what went with a message.

import { Camera, FileText, Paperclip, X } from "lucide-react";
import { useRef } from "react";
import { useT } from "../i18n";
import { fileSize } from "../lib/format";
import type { Outgoing } from "./outgoing";

/** "Send a photo" (the camera or the gallery) and "Send a file". */
export function AttachButtons({
  disabled,
  pick,
}: {
  disabled: boolean;
  pick(files: File[], asPhoto: boolean): void;
}) {
  const c = useT().phone.chats.composer;
  const photos = useRef<HTMLInputElement>(null);
  const files = useRef<HTMLInputElement>(null);
  const button =
    "inline-flex h-12 w-11 flex-none items-center justify-center rounded-xl border border-line-strong bg-panel text-ink-soft disabled:opacity-50";
  const take = (asPhoto: boolean) => (event: React.ChangeEvent<HTMLInputElement>) => {
    const picked = [...(event.target.files ?? [])];
    event.target.value = "";
    if (picked.length > 0) {
      pick(picked, asPhoto);
    }
  };
  return (
    <>
      <button
        type="button"
        aria-label={c.photo}
        disabled={disabled}
        onClick={() => photos.current?.click()}
        className={button}
      >
        <Camera aria-hidden size={20} />
      </button>
      <button
        type="button"
        aria-label={c.file}
        disabled={disabled}
        onClick={() => files.current?.click()}
        className={button}
      >
        <Paperclip aria-hidden size={20} />
      </button>
      <input
        ref={photos}
        type="file"
        accept="image/*"
        multiple
        hidden
        data-testid="photo-picker"
        onChange={take(true)}
      />
      <input
        ref={files}
        type="file"
        multiple
        hidden
        data-testid="file-picker"
        onChange={take(false)}
      />
    </>
  );
}

/** What was picked and goes with the next message, each one can be taken out. */
export function Chosen({ files, remove }: { files: Outgoing[]; remove(index: number): void }) {
  const c = useT().phone.chats.composer;
  if (files.length === 0) {
    return null;
  }
  return (
    <ul aria-label={c.chosen} className="flex gap-2 overflow-x-auto pb-1">
      {files.map((file, index) => (
        // The same file may be picked twice, so the place is part of the key.
        <li
          // biome-ignore lint/suspicious/noArrayIndexKey: picked files have no id of their own
          key={`${index}-${file.name}`}
          className="flex max-w-56 flex-none items-center gap-2 rounded-xl border border-line bg-panel py-1 pr-1 pl-1.5"
        >
          {file.thumb ? (
            <img src={file.thumb} alt="" className="h-10 w-10 flex-none rounded-lg object-cover" />
          ) : (
            <FileText aria-hidden size={20} className="ml-1 flex-none text-muted" />
          )}
          <span className="flex min-w-0 flex-col">
            <span className="truncate text-sm">{file.name}</span>
            <span className="text-muted text-xs">{fileSize(file.blob.size)}</span>
          </span>
          <button
            type="button"
            aria-label={c.remove(file.name)}
            onClick={() => remove(index)}
            className="inline-flex h-9 w-9 flex-none items-center justify-center rounded-lg text-muted"
          >
            <X aria-hidden size={16} />
          </button>
        </li>
      ))}
    </ul>
  );
}

/** The names of the files that went with a message. */
export function Attached({ names }: { names: string[] }) {
  const c = useT().phone.chats;
  return (
    <ul aria-label={c.attached(names.length)} className="flex flex-col items-end gap-1">
      {names.map((name, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: two files may have the same name
        <li key={`${index}-${name}`} className="flex items-center gap-1.5 text-ink-soft text-sm">
          <Paperclip aria-hidden size={14} className="flex-none" />
          <span className="truncate">{name}</span>
        </li>
      ))}
    </ul>
  );
}
