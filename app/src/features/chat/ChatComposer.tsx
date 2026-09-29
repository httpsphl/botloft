// Writing to the bot: text, and files picked, pasted or dropped on the
// chat. Enter sends and Shift+Enter starts a new line (spec 15.1).

import { ArrowUp, File as FileIcon, Paperclip, X } from "lucide-react";
import {
  type ClipboardEvent,
  type FormEvent,
  type KeyboardEvent,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { fileSize } from "../../lib/format";
import { type Bot, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { rememberImage } from "./images";
import type { Files, PendingFile } from "./useFiles";

const MAX_HEIGHT_PX = 240;

function Chip({ file, onRemove }: { file: PendingFile; onRemove(): void }) {
  const t = useT();
  return (
    <li className="relative flex h-14 items-center gap-2 rounded-md border border-line bg-canvas pr-7 pl-1.5">
      {file.preview ? (
        <img src={file.preview} alt="" className="h-11 w-11 rounded-[3px] object-cover" />
      ) : (
        <FileIcon aria-hidden size={20} className="mx-1 text-muted" />
      )}
      <div className="min-w-0 max-w-40">
        <p className="truncate font-medium text-xs" title={file.name}>
          {file.name}
        </p>
        <p className="text-muted text-xs">{fileSize(file.size)}</p>
      </div>
      <button
        type="button"
        aria-label={t.chat.composer.remove(file.name)}
        onClick={onRemove}
        className="absolute top-1 right-1 grid h-5 w-5 place-items-center rounded-full text-muted hover:bg-sunken hover:text-ink"
      >
        <X aria-hidden size={12} />
      </button>
    </li>
  );
}

export function ChatComposer({
  bot,
  files,
  stopped,
  onSent,
}: {
  bot: Bot;
  files: Files;
  /** The bot is paused or its crew is. */
  stopped: boolean;
  onSent(): void;
}) {
  const api = useApi();
  const t = useT();
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const field = useRef<HTMLTextAreaElement>(null);
  const picker = useRef<HTMLInputElement>(null);
  const fieldId = useId();
  const tooLong = text.length > FIELD_LIMITS.message;
  const empty = !text.trim() && files.files.length === 0;

  // Grows with the text up to a limit, then scrolls.
  useLayoutEffect(() => {
    const element = field.current;
    if (element) {
      element.style.height = "auto";
      if (text) {
        element.style.height = `${Math.min(element.scrollHeight, MAX_HEIGHT_PX)}px`;
      }
    }
  }, [text]);

  const send = async (event?: FormEvent) => {
    event?.preventDefault();
    if (empty || tooLong || busy) {
      return;
    }
    setBusy(true);
    files.setError(null);
    const pending = files.files;
    try {
      const attachments = pending.map(({ name, mediaType, data }) => ({ name, mediaType, data }));
      const message = await api.call("messages.send", {
        botId: bot.id,
        body: text,
        ...(attachments.length > 0 ? { attachments } : {}),
      });
      // The daemon keeps the order it was given.
      message.attachments.forEach((attachment, index) => {
        const preview = pending[index]?.preview;
        if (preview) {
          rememberImage(attachment.id, preview);
        }
      });
      setText("");
      files.clear();
      onSent();
    } catch (failure) {
      files.setError(errorText(failure));
    } finally {
      setBusy(false);
      field.current?.focus();
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      void send();
    }
  };

  const onPaste = (event: ClipboardEvent<HTMLTextAreaElement>) => {
    const pasted = [...event.clipboardData.files];
    if (pasted.length > 0) {
      event.preventDefault();
      void files.add(pasted);
    }
  };

  return (
    <form onSubmit={send} className="shrink-0 px-5 pt-2 pb-4">
      {stopped && <p className="mb-1.5 text-muted text-xs">{t.chat.composer.paused(bot.name)}</p>}
      <div className="rounded-lg border border-line-strong bg-panel focus-within:border-accent">
        {files.files.length > 0 && (
          <ul
            aria-label={t.chat.composer.filesToSend}
            className="flex flex-wrap gap-2 px-2.5 pt-2.5"
          >
            {files.files.map((file) => (
              <Chip key={file.key} file={file} onRemove={() => files.remove(file.key)} />
            ))}
          </ul>
        )}
        <label htmlFor={fieldId} className="sr-only">
          {t.chat.composer.label(bot.name)}
        </label>
        <textarea
          id={fieldId}
          ref={field}
          value={text}
          rows={1}
          onChange={(event) => setText(event.target.value)}
          onKeyDown={onKeyDown}
          onPaste={onPaste}
          placeholder={t.chat.composer.placeholder(bot.name)}
          className="block max-h-60 w-full resize-none bg-transparent px-3.5 pt-3 pb-1 leading-relaxed outline-none placeholder:text-muted"
        />
        <div className="flex items-center gap-2 px-2 pb-2">
          <button
            type="button"
            aria-label={t.chat.composer.attach}
            title={t.chat.composer.attachHint}
            onClick={() => picker.current?.click()}
            className="grid h-8 w-8 place-items-center rounded-[3px] text-muted hover:bg-sunken hover:text-ink"
          >
            <Paperclip aria-hidden size={16} />
          </button>
          <input
            ref={picker}
            type="file"
            multiple
            hidden
            data-testid="file-picker"
            onChange={(event) => {
              void files.add([...(event.target.files ?? [])]);
              event.target.value = "";
            }}
          />
          <span className={`flex-1 text-xs ${tooLong ? "text-danger" : "text-muted"}`}>
            {tooLong
              ? t.chat.composer.tooLong(text.length, FIELD_LIMITS.message)
              : t.chat.composer.keys}
          </span>
          <button
            type="submit"
            aria-label={t.chat.composer.send}
            disabled={empty || tooLong || busy}
            className="grid h-8 w-8 place-items-center rounded-full bg-ink text-canvas hover:bg-ink-soft disabled:opacity-35"
          >
            <ArrowUp aria-hidden size={16} strokeWidth={2.25} />
          </button>
        </div>
      </div>
      {files.error && (
        <p role="alert" className="mt-1.5 text-danger text-sm">
          {files.error}
        </p>
      )}
    </form>
  );
}
