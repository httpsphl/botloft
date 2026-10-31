// The island's Chat tab (spec 31.4): one agent, picked from a menu, its
// last words and the owner's, the answer as it comes, and a field to send
// the next message, with files dropped on the island.

import { ArrowUp, ChevronDown, Paperclip, X } from "lucide-react";
import { type CSSProperties, type FormEvent, type ReactNode, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { toolSummary } from "../features/chat/toolSummary";
import type { PendingFile } from "../features/chat/useFiles";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Bot, BotId, ChatItem, ToolItem } from "../lib/protocol.gen";
import { useTalk } from "./useTalk";

/** How many lines of talk the tab shows. */
const SHOWN = 6;

type Line =
  | { kind: "owner" | "agent"; id: string; text: string }
  | { kind: "tools"; id: string; tools: ToolItem[] };

/** The talk as lines: the owner's messages, the agent's replies, and each
 *  run of tool calls folded into one. */
function linesOf(items: ChatItem[]): Line[] {
  const lines: Line[] = [];
  for (const item of items) {
    const body = item.body;
    if (body.kind === "inbound" && body.message.fromKind === "owner") {
      lines.push({ kind: "owner", id: item.id, text: body.message.body });
    } else if (body.kind === "reply") {
      lines.push({ kind: "agent", id: item.id, text: body.text });
    } else if (body.kind === "tool") {
      const last = lines.at(-1);
      if (last?.kind === "tools") {
        last.tools.push(body);
      } else {
        lines.push({ kind: "tools", id: item.id, tools: [body] });
      }
    }
  }
  return lines.slice(-SHOWN);
}

function Picker({ bots, bot, onPick }: { bots: Bot[]; bot: Bot; onPick: (id: BotId) => void }) {
  const t = useT();
  const [open, setOpen] = useState(false);
  return (
    <div className="relative flex w-32 shrink-0 flex-col items-center gap-1.5 pt-2 text-center">
      <span className="island-glow" style={{ "--glow": bot.color } as CSSProperties}>
        <BotAvatar
          color={bot.color}
          size={56}
          mood={bot.state === "busy" ? "working" : "idle"}
          still
        />
      </span>
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-label={t.island.chat.pick}
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1 font-semibold text-[15px]"
      >
        {bot.name}
        <ChevronDown aria-hidden size={14} className="island-muted" />
      </button>
      {open && (
        <div
          role="listbox"
          aria-label={t.island.chat.pick}
          className="island-card absolute top-full left-0 z-10 mt-1 max-h-60 w-48 overflow-auto p-1 text-left shadow-lg"
        >
          {bots.map((other) => (
            <button
              key={other.id}
              type="button"
              role="option"
              aria-selected={other.id === bot.id}
              onClick={() => {
                onPick(other.id);
                setOpen(false);
              }}
              className="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-[13px] hover:bg-[var(--island-tab)]"
            >
              <BotAvatar color={other.color} size={20} mood="idle" still />
              <span className="truncate">{other.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function Lines({ lines, writing }: { lines: Line[]; writing: string }) {
  const t = useT();
  const shown: ReactNode[] = lines.map((line) => {
    if (line.kind === "owner") {
      return (
        <span
          key={line.id}
          className="max-w-[85%] self-end rounded-2xl bg-[var(--island-tab)] px-3 py-1.5"
        >
          <span className="line-clamp-3 whitespace-pre-wrap break-words">{line.text}</span>
        </span>
      );
    }
    if (line.kind === "tools") {
      return (
        <span key={line.id} className="island-muted text-xs">
          {toolSummary(line.tools, t.chat.tools.group)}
        </span>
      );
    }
    return (
      <span
        key={line.id}
        className="line-clamp-4 max-w-[90%] whitespace-pre-wrap break-words text-[var(--island-ink)]"
      >
        {line.text}
      </span>
    );
  });
  return (
    <div className="flex flex-col gap-2 text-[13px]">
      {shown}
      {writing.trim() !== "" && (
        <span className="line-clamp-4 max-w-[90%] whitespace-pre-wrap break-words text-[var(--island-ink)]">
          {writing}
          <span className="animate-pulse">▍</span>
        </span>
      )}
    </div>
  );
}

export function ChatTab({
  api,
  bots,
  botId,
  onPick,
  files,
  onRemoveFile,
  fileError,
  onSent,
  onOpen,
}: {
  api: BotloftApi;
  bots: Bot[];
  botId: BotId | null;
  onPick: (id: BotId) => void;
  files: PendingFile[];
  onRemoveFile: (key: string) => void;
  fileError: string | null;
  onSent: () => void;
  onOpen: (botId: BotId) => void;
}) {
  const t = useT();
  const bot = bots.find((have) => have.id === botId) ?? bots[0];
  const { items, writing } = useTalk(api, bot?.id ?? null);
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);
  if (!bot) {
    return null;
  }
  const send = (event: FormEvent) => {
    event.preventDefault();
    if (busy || (text.trim() === "" && files.length === 0)) {
      return;
    }
    setBusy(true);
    setFailed(false);
    const attachments = files.map(({ name, mediaType, data }) => ({ name, mediaType, data }));
    api
      .call("messages.send", {
        botId: bot.id,
        body: text.trim(),
        ...(attachments.length > 0 ? { attachments } : {}),
      })
      .then(
        () => {
          setText("");
          onSent();
        },
        () => setFailed(true),
      )
      .finally(() => setBusy(false));
  };
  return (
    <div className="island-card flex gap-4 p-4">
      <Picker bots={bots} bot={bot} onPick={onPick} />
      <div className="flex min-w-0 flex-1 flex-col gap-2.5">
        <Lines lines={linesOf(items)} writing={writing} />
        {files.length > 0 && (
          <ul aria-label={t.island.chat.files} className="flex flex-wrap gap-1.5">
            {files.map((file) => (
              <li
                key={file.key}
                className="island-sunk flex items-center gap-1.5 py-1 pr-1 pl-2 text-xs"
              >
                <Paperclip aria-hidden size={12} className="island-muted" />
                <span className="max-w-40 truncate">{file.name}</span>
                <button
                  type="button"
                  aria-label={t.island.chat.removeFile(file.name)}
                  onClick={() => onRemoveFile(file.key)}
                >
                  <X aria-hidden size={12} />
                </button>
              </li>
            ))}
          </ul>
        )}
        {(fileError || failed) && (
          <p className="text-[12px] text-[var(--island-danger)]">
            {fileError ?? t.island.card.failed}
          </p>
        )}
        <form
          onSubmit={send}
          className="island-field mt-auto flex items-center gap-2 py-1.5 pr-1.5 pl-4"
        >
          <input
            value={text}
            onChange={(event) => setText(event.target.value)}
            aria-label={t.island.chat.label(bot.name)}
            placeholder={t.island.chat.placeholder(bot.name)}
            className="min-w-0 flex-1 bg-transparent text-[13px] outline-none"
          />
          <button
            type="submit"
            aria-label={t.island.chat.send}
            disabled={busy || (text.trim() === "" && files.length === 0)}
            className="grid size-7 place-items-center rounded-full bg-white text-black disabled:opacity-50"
          >
            <ArrowUp aria-hidden size={15} />
          </button>
        </form>
        <div className="flex">
          <button
            type="button"
            className="island-button ml-auto"
            data-kind="quiet"
            onClick={() => onOpen(bot.id)}
          >
            {t.island.card.openInApp}
          </button>
        </div>
      </div>
    </div>
  );
}
