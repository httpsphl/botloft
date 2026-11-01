// One conversation (spec 28.12): the newest items at the bottom, older ones
// a tap away, the reply as it is written, and the box to write in.

import { useEffect, useRef, useState } from "react";
import { useT } from "../i18n";
import { AttachButtons, Chosen } from "./Attach";
import { ChatItem, PendingItem } from "./ChatItems";
import type { PhoneApi, PhoneState } from "./client";
import { asIs, FILE_MAX, FILES_MAX, type Outgoing, photo } from "./outgoing";
import { BotDot, PhoneButton, PhoneMarkdown } from "./parts";
import { SEND_MAX } from "./talk";

/** How far from the end still counts as reading the end. */
const FOLLOW_PX = 120;

export function ChatView({ api, state, back }: { api: PhoneApi; state: PhoneState; back(): void }) {
  const c = useT().phone.chats;
  const { convo } = state;
  const line = state.chats.find((one) => one.botId === convo?.botId);
  const [text, setText] = useState("");
  const [tooLong, setTooLong] = useState(false);
  const [files, setFiles] = useState<Outgoing[]>([]);
  /** Why a file was left out, if one was. */
  const [fileNote, setFileNote] = useState<string | null>(null);
  const [preparing, setPreparing] = useState(false);
  const column = useRef<HTMLDivElement>(null);
  /** The owner is at the end of the conversation, so what comes is followed. */
  const atEnd = useRef(true);
  const failed = convo?.pending.find((pending) => pending.status === "failed");

  // Whatever makes the conversation taller (a new item, the reply as it is
  // written, "working…", a card) keeps its end in view for whoever is
  // reading the end; someone who scrolled up is left where they are.
  const open = convo !== null;
  useEffect(() => {
    const node = column.current;
    if (!open || !node) {
      return;
    }
    const onScroll = () => {
      const page = document.documentElement;
      atEnd.current = page.scrollHeight - window.scrollY - window.innerHeight < FOLLOW_PX;
    };
    const follow = () => {
      if (atEnd.current) {
        window.scrollTo({ top: document.documentElement.scrollHeight });
      }
    };
    atEnd.current = true;
    window.addEventListener("scroll", onScroll, { passive: true });
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(follow);
    observer?.observe(node);
    follow();
    return () => {
      window.removeEventListener("scroll", onScroll);
      observer?.disconnect();
    };
  }, [open]);

  // A message that did not go comes back to the box, if the box is empty,
  // and so do its files.
  const failedText = failed?.text;
  const failedFiles = failed?.files;
  useEffect(() => {
    if (failedText !== undefined) {
      setText((now) => (now === "" ? failedText : now));
    }
  }, [failedText]);
  useEffect(() => {
    if (failedFiles) {
      setFiles((now) => (now.length === 0 ? failedFiles : now));
    }
  }, [failedFiles]);

  if (!convo) {
    return null;
  }
  const name = line?.name ?? "";
  const pick = async (picked: File[], asPhoto: boolean) => {
    setFileNote(null);
    const room = FILES_MAX - files.length;
    if (picked.length > room) {
      setFileNote(c.composer.tooMany);
    }
    setPreparing(asPhoto);
    const ready: Outgoing[] = [];
    for (const file of picked.slice(0, Math.max(0, room))) {
      const out = asPhoto ? await photo(file) : asIs(file);
      if (out.blob.size > FILE_MAX) {
        setFileNote(c.composer.tooBig(file.name));
      } else {
        ready.push(out);
      }
    }
    setPreparing(false);
    setFiles((now) => [...now, ...ready].slice(0, FILES_MAX));
  };
  const submit = async () => {
    const bytes = new TextEncoder().encode(text.trim()).length;
    if (bytes === 0 && files.length === 0) {
      return;
    }
    if (bytes > SEND_MAX) {
      setTooLong(true);
      return;
    }
    setTooLong(false);
    setFileNote(null);
    atEnd.current = true;
    const [body, chosen] = [text, files];
    setText("");
    setFiles([]);
    await api.write(body, chosen);
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <header className="sticky top-0 z-10 -mx-4 flex items-center gap-3 border-line border-b bg-canvas px-4 py-2">
        <PhoneButton onClick={back} className="flex-none">
          {c.back}
        </PhoneButton>
        {line && <BotDot name={line.name} color={line.color} state={line.state} />}
        <div className="flex min-w-0 flex-col">
          <h1 className="truncate font-semibold text-lg">{name}</h1>
          {line && <span className="truncate text-muted text-xs">{c.inCrew(line.crew)}</span>}
        </div>
      </header>

      <div ref={column} className="flex flex-1 flex-col gap-3">
        {convo.more && (
          <PhoneButton
            className="flex-none"
            onClick={() => {
              atEnd.current = false;
              void api.olderItems();
            }}
            disabled={convo.older}
          >
            {convo.older ? c.loadingOlder : c.older}
          </PhoneButton>
        )}
        {!convo.loaded && <p className="text-center text-muted text-sm">{c.loadingChat}</p>}
        {convo.loaded && convo.items.length === 0 && convo.pending.length === 0 && (
          <p className="py-8 text-center text-muted text-sm">{c.nothing}</p>
        )}
        {convo.items.map((item) => (
          <ChatItem key={item.id} item={item} api={api} state={state} />
        ))}
        {convo.pending.map((pending) => (
          <PendingItem key={pending.clientId} pending={pending} />
        ))}
        {convo.live !== "" && (
          <div className="self-start rounded-2xl border border-line bg-panel px-3 py-2">
            <PhoneMarkdown>{convo.live}</PhoneMarkdown>
          </div>
        )}
        {convo.working && convo.live === "" && (
          <p role="status" className="text-muted text-sm">
            {c.busy(name)}
          </p>
        )}
      </div>

      <form
        className="sticky bottom-0 flex flex-col gap-2 bg-canvas pt-2 pb-[max(0.5rem,env(safe-area-inset-bottom))]"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        {tooLong && (
          <p role="alert" className="text-danger text-sm">
            {c.composer.tooLong}
          </p>
        )}
        {fileNote && (
          <p role="alert" className="text-danger text-sm">
            {fileNote}
          </p>
        )}
        {preparing && (
          <p role="status" className="text-muted text-sm">
            {c.composer.preparing}
          </p>
        )}
        <Chosen files={files} remove={(at) => setFiles((now) => now.filter((_, i) => i !== at))} />
        <div className="flex items-end gap-2">
          <AttachButtons disabled={preparing || files.length >= FILES_MAX} pick={pick} />
          <textarea
            aria-label={c.composer.label(name)}
            placeholder={c.composer.placeholder}
            value={text}
            rows={2}
            onChange={(event) => {
              setText(event.target.value);
              setTooLong(false);
            }}
            className="max-h-40 min-h-12 min-w-0 flex-1 resize-none rounded-xl border border-line-strong bg-panel px-3 py-2 text-base text-ink"
          />
          <PhoneButton
            look="primary"
            type="submit"
            disabled={
              (text.trim() === "" && files.length === 0) || preparing || state.link !== "online"
            }
            className="flex-none"
          >
            {c.composer.send}
          </PhoneButton>
        </div>
      </form>
    </div>
  );
}
