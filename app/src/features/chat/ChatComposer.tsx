// Writing to the bot: text, and files picked, pasted or dropped on the
// chat. Enter sends and Shift+Enter starts a new line, or, if the owner
// chose so in Settings, Ctrl+Enter sends and Enter starts a line (spec
// 15.1).

import { ArrowUp, File as FileIcon, Paperclip, X } from "lucide-react";
import {
  type ClipboardEvent,
  type FormEvent,
  type KeyboardEvent,
  memo,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { fileSize } from "../../lib/format";
import { type Bot, FIELD_LIMITS } from "../../lib/protocol.gen";
import { prefs, usePref } from "../../shell/prefs";
import { useApi } from "../../store/context";
import { ContextMeter } from "./ContextMeter";
import { draftOf, keepDraft } from "./drafts";
import { EffortPicker } from "./EffortPicker";
import { rememberImage } from "./images";
import { ModelPicker } from "./ModelPicker";
import { ModePicker } from "./ModePicker";
import type { Files, PendingFile } from "./useFiles";

const MAX_HEIGHT_PX = 240;

function Chip({ file, onRemove }: { file: PendingFile; onRemove(): void }) {
  const t = useT();
  return (
    <li className="relative flex h-14 items-center gap-2 rounded-xl border border-line bg-canvas pr-7 pl-1.5">
      {file.preview ? (
        <img src={file.preview} alt="" className="h-11 w-11 rounded-lg object-cover" />
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

export const ChatComposer = memo(function ChatComposer({
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
  const botId = bot.id;
  const [text, setShown] = useState(() => draftOf(botId));
  const setText = (value: string) => {
    setShown(value);
    keepDraft(botId, value);
  };
  const [busy, setBusy] = useState(false);
  /** A new mode, model or effort, or a compaction, that waits for the bot to finish what it is doing. */
  const [later, setLater] = useState<string | null>(null);
  const field = useRef<HTMLTextAreaElement>(null);
  const picker = useRef<HTMLInputElement>(null);
  const fieldId = useId();
  const tooLong = text.length > FIELD_LIMITS.message;
  const empty = !text.trim() && files.files.length === 0;

  const working = bot.state === "busy" || bot.state === "needs_approval";
  useEffect(() => {
    if (!working) {
      setLater(null);
    }
  }, [working]);

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

  const enterSends = usePref(prefs.enterSends);
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key !== "Enter" || event.nativeEvent.isComposing) {
      return;
    }
    const withCtrl = event.ctrlKey || event.metaKey;
    if (enterSends ? !event.shiftKey : withCtrl) {
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
    <form onSubmit={send} className="shrink-0 px-5 pt-2 pb-5">
      {stopped && (
        <p className="mb-1.5 px-2 text-muted text-xs">{t.chat.composer.paused(bot.name)}</p>
      )}
      {later && (
        <p role="status" className="mb-1.5 animate-fade px-2 text-muted text-xs">
          {later}
        </p>
      )}
      <div className="@container rounded-2xl border border-line-strong bg-panel shadow-sm transition-colors focus-within:border-muted">
        {files.files.length > 0 && (
          <ul aria-label={t.chat.composer.filesToSend} className="flex flex-wrap gap-2 px-3 pt-3">
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
          className="block max-h-60 w-full resize-none bg-transparent px-4 pt-3.5 pb-1.5 leading-relaxed outline-none placeholder:text-muted"
        />
        {/* The meter and the effort open over the row's right edge. The row
            never wraps: in a narrow chat the labels give way instead. */}
        <div className="relative flex items-center gap-1 px-2.5 pb-2.5">
          <button
            type="button"
            aria-label={t.chat.composer.attach}
            title={t.chat.composer.attachHint}
            onClick={() => picker.current?.click()}
            className="grid h-8 w-8 shrink-0 place-items-center rounded-full text-muted hover:bg-sunken hover:text-ink"
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
          <ModePicker bot={bot} onLater={setLater} />
          <span
            className={`min-w-0 flex-1 truncate px-1 text-right text-xs ${tooLong ? "text-danger" : "text-muted"}`}
          >
            {tooLong
              ? t.chat.composer.tooLong(text.length, FIELD_LIMITS.message)
              : text && (enterSends ? t.chat.composer.keys : t.chat.composer.keysWithCtrl)}
          </span>
          <div className="ml-auto flex min-w-0 max-w-full items-center gap-1">
            <ContextMeter bot={bot} onLater={setLater} />
            <EffortPicker bot={bot} onLater={setLater} />
            <ModelPicker bot={bot} onLater={setLater} />
            <button
              type="submit"
              aria-label={t.chat.composer.send}
              disabled={empty || tooLong || busy}
              className="group/send grid h-8 w-8 shrink-0 place-items-center rounded-full bg-accent text-canvas transition-[opacity,transform,background-color] duration-150 hover:opacity-90 active:scale-90 disabled:bg-line-strong disabled:opacity-60"
            >
              <ArrowUp
                aria-hidden
                size={16}
                strokeWidth={2.25}
                className="transition-transform duration-200 group-hover/send:-translate-y-0.5"
              />
            </button>
          </div>
        </div>
      </div>
      {files.error && (
        <p role="alert" className="mt-1.5 text-danger text-sm">
          {files.error}
        </p>
      )}
    </form>
  );
});
