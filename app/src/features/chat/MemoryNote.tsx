// A line in the bot's turn saying it wrote down something it learned (spec
// 15.3), with whose memory it was: its own, or the crew's. The arrow opens
// what changed; the files button opens the whole memory.

import { BookMarked, ChevronRight, Files } from "lucide-react";
import { useContext, useState } from "react";
import { useT } from "../../i18n";
import type { ToolItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { ShowFile } from "../files/showFile";
import { type MemoryChange, type MemoryOwner, memoryChange } from "./memory";

function Lines({ lines, sign }: { lines: string[]; sign: "-" | "+" }) {
  const tone = sign === "-" ? "bg-danger/10 text-danger" : "bg-ok/10 text-ok";
  return lines.map((line, index) => (
    // Lines repeat and never move: their place is their key.
    // biome-ignore lint/suspicious/noArrayIndexKey: a fixed list
    <span key={index} className={`block px-2 ${tone}`}>
      <span aria-hidden className="mr-2 select-none">
        {sign}
      </span>
      <span className="text-ink">{line || " "}</span>
    </span>
  ));
}

function Change({ change }: { change: MemoryChange }) {
  const t = useT().chat.memory;
  const block =
    "max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg border border-line bg-sunken py-1.5 font-mono text-xs leading-relaxed";
  if (change.kind === "write") {
    return (
      <>
        <p className="text-muted text-xs">{t.now}</p>
        <pre className={`${block} px-2.5`}>{change.text}</pre>
      </>
    );
  }
  return (
    <figure className={block} aria-label={t.changed}>
      {change.hunks.map((hunk, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: a fixed list
        <span key={index} className={`block ${index ? "mt-1.5 border-line border-t pt-1.5" : ""}`}>
          <Lines lines={hunk.removed} sign="-" />
          <Lines lines={hunk.added} sign="+" />
        </span>
      ))}
    </figure>
  );
}

export function MemoryNote({
  tool,
  createdAt,
  owner,
  name,
  color,
}: {
  tool: ToolItem;
  createdAt: number;
  owner: MemoryOwner;
  /** The bot's name, or the crew's. */
  name: string;
  /** The bot's color; the crew has none. */
  color?: string | undefined;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const showFile = useContext(ShowFile);
  const arrival = useArrival(createdAt);
  const change = memoryChange(tool);
  const words = owner === "bot" ? t.chat.memory.bot : t.chat.memory.crew;
  const label = (
    <>
      <BookMarked aria-hidden size={14} className="shrink-0" />
      <span className="shrink-0">{words}</span>
      <span className="flex min-w-0 items-center gap-1.5 rounded-full bg-sunken px-2 py-0.5 font-medium text-ink text-xs">
        {color && (
          <span
            aria-hidden
            className="size-2 shrink-0 rounded-full"
            style={{ background: color }}
          />
        )}
        <span className="truncate">{name}</span>
      </span>
    </>
  );
  const line =
    "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-left text-muted text-sm";
  return (
    <div className={`-mx-1.5 flex flex-col ${arrival}`}>
      <div className="flex items-center gap-1 self-start">
        {change ? (
          <button
            type="button"
            aria-expanded={open}
            onClick={() => setOpen(!open)}
            className={`${line} transition-colors hover:bg-sunken hover:text-ink`}
          >
            {label}
            <ChevronRight
              aria-hidden
              size={13}
              className={`shrink-0 transition-transform ${open ? "rotate-90" : ""}`}
            />
          </button>
        ) : (
          <p className={line}>{label}</p>
        )}
        {showFile && tool.file && (
          <button
            type="button"
            title={t.files.showInPanel}
            aria-label={`${t.files.showInPanel}: ${t.chat.memory.file}`}
            onClick={() => showFile(tool.file as string)}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <Files aria-hidden size={14} />
          </button>
        )}
      </div>
      {open && change && (
        <div className="mt-1 mb-2 ml-7 flex animate-rise flex-col gap-1.5" data-selectable>
          <Change change={change} />
        </div>
      )}
    </div>
  );
}
