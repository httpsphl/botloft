// What the bot did with its tools: one compact line per call (icon, tool,
// summary, state) that opens to show the input and the output (spec 15.3).

import {
  Check,
  ChevronRight,
  FilePen,
  FileSearch,
  FileText,
  Globe,
  ListChecks,
  ListTodo,
  LoaderCircle,
  type LucideIcon,
  Send,
  SquareTerminal,
  UserPlus,
  Users,
  Wrench,
  X,
} from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";

const ICONS: Record<string, LucideIcon> = {
  Bash: SquareTerminal,
  PowerShell: SquareTerminal,
  Read: FileText,
  Edit: FilePen,
  MultiEdit: FilePen,
  Write: FilePen,
  NotebookEdit: FilePen,
  Grep: FileSearch,
  Glob: FileSearch,
  WebFetch: Globe,
  WebSearch: Globe,
  TodoWrite: ListChecks,
  ExitPlanMode: ListTodo,
  Task: Users,
  Agent: Users,
  send_message: Send,
  suggest_bot: UserPlus,
};

/** `mcp__botloft__send_message` reads as `send_message`. */
export function toolLabel(name: string): string {
  return name.split("__").at(-1) ?? name;
}

/** JSON indented for reading; anything else as it came. */
export function pretty(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return text;
  }
}

function Status({ status }: { status: ToolItem["status"] }) {
  const t = useT();
  switch (status) {
    case "running":
      return (
        <LoaderCircle
          aria-label={t.chat.tools.running}
          size={13}
          className="shrink-0 animate-spin text-work"
        />
      );
    case "done":
      return <Check aria-label={t.chat.tools.done} size={13} className="shrink-0 text-ok" />;
    case "failed":
      return <X aria-label={t.chat.tools.failed} size={13} className="shrink-0 text-danger" />;
  }
}

function ToolLine({ tool, createdAt }: { tool: ToolItem; createdAt: number }) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const arrival = useArrival(createdAt);
  const label = toolLabel(tool.name);
  const Icon = ICONS[label] ?? Wrench;
  const block =
    "max-h-64 overflow-auto whitespace-pre-wrap break-all rounded-lg border border-line bg-sunken px-2.5 py-1.5 font-mono text-xs leading-relaxed";
  return (
    <li className={arrival}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="group flex w-full min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-left text-sm transition-colors hover:bg-sunken"
      >
        <Icon aria-hidden size={14} className="shrink-0 text-muted" />
        <span className="shrink-0 font-medium">{label}</span>
        <span className="min-w-0 flex-1 truncate font-mono text-muted text-xs" title={tool.summary}>
          {tool.summary}
        </span>
        <Status status={tool.status} />
        <ChevronRight
          aria-hidden
          size={13}
          className={`shrink-0 text-muted transition-transform ${open ? "rotate-90" : ""}`}
        />
      </button>
      {open && (
        <div className="mt-1 mb-2 ml-7 flex animate-rise flex-col gap-1.5" data-selectable>
          <p className="text-muted text-xs">{t.chat.tools.input}</p>
          <pre className={block}>{pretty(tool.input)}</pre>
          {tool.output !== null && (
            <>
              <p className={`text-xs ${tool.status === "failed" ? "text-danger" : "text-muted"}`}>
                {tool.status === "failed" ? t.chat.tools.error : t.chat.tools.output}
              </p>
              <pre className={block}>{tool.output}</pre>
            </>
          )}
        </div>
      )}
    </li>
  );
}

export function ToolLines({ items }: { items: ChatItem[] }) {
  const t = useT();
  return (
    <ul aria-label={t.chat.tools.label} className="-mx-1.5 flex flex-col">
      {items.map((item) =>
        item.body.kind === "tool" ? (
          <ToolLine key={item.id} tool={item.body} createdAt={item.createdAt} />
        ) : null,
      )}
    </ul>
  );
}
