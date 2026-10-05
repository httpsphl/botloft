// What the bot did with its tools: one compact line per call (icon, tool,
// what it is about, state) that opens to show the input, or the command,
// and the output (spec 15.3).

import {
  AlarmClock,
  Check,
  ChevronRight,
  FilePen,
  FileSearch,
  Files,
  FileText,
  Globe,
  Hand,
  LayoutTemplate,
  ListChecks,
  ListTodo,
  LoaderCircle,
  type LucideIcon,
  Monitor,
  Network,
  Send,
  SquareTerminal,
  UserPen,
  UserPlus,
  Users,
  Wrench,
  X,
} from "lucide-react";
import { useContext, useState } from "react";
import { useT } from "../../i18n";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { isBrowserTool, ShowBrowser } from "../browser/showBrowser";
import { isDesktopTool, ShowDesktop } from "../desktop/showDesktop";
import { ShowFile } from "../files/showFile";
import { isScreenFile, ShowScreen } from "../screens/showScreen";
import { ShowTerminal } from "../terminal/showTerminal";
import { commandOf, isCommand } from "./command";
import { ReadPicture, ReadThumb, useReadImage } from "./readImage";
import { toolDetail, toolKey, toolTitle } from "./toolNames";

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
  change_bot: UserPen,
  ask_crew_access: Network,
  schedule_routine: AlarmClock,
  my_routines: AlarmClock,
  change_routine: AlarmClock,
  delete_routine: AlarmClock,
  browser_ask_owner: Hand,
};

/** The browser tools share the globe (spec 21.4), the desktop's the screen (24.6). */
function iconOf(label: string): LucideIcon {
  if (ICONS[label]) {
    return ICONS[label];
  }
  if (label.startsWith("browser_")) {
    return Globe;
  }
  return label.startsWith("desktop_") ? Monitor : Wrench;
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

function ToolLine({
  tool,
  botId,
  createdAt,
}: {
  tool: ToolItem;
  botId: string;
  createdAt: number;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const showFile = useContext(ShowFile);
  const showBrowser = useContext(ShowBrowser);
  const showScreen = useContext(ShowScreen);
  const showTerminal = useContext(ShowTerminal);
  const showDesktop = useContext(ShowDesktop);
  const arrival = useArrival(createdAt);
  const picture = useReadImage(botId, tool);
  const title = toolTitle(tool.name, t.tools);
  // A command reads in the bot's own words, when it gave them; the command
  // itself is one click away, below.
  const said = tool.explanation;
  const detail = said || toolDetail(tool, t.tools);
  const command = isCommand(tool.name) ? commandOf(tool.input) : null;
  const Icon = iconOf(toolKey(tool.name));
  const block =
    "max-h-64 overflow-auto whitespace-pre-wrap break-all rounded-lg border border-line bg-sunken px-2.5 py-1.5 font-mono text-xs leading-relaxed";
  return (
    <li className={arrival}>
      <div className="flex items-center gap-1">
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="group flex min-w-0 flex-1 items-center gap-2 rounded-lg px-1.5 py-1 text-left text-sm transition-colors hover:bg-sunken"
        >
          <Icon aria-hidden size={14} className="shrink-0 text-muted" />
          <span className="shrink-0 font-medium">{title}</span>
          <span
            className={`min-w-0 flex-1 truncate text-muted text-xs ${said ? "" : "font-mono"}`}
            title={detail}
          >
            {detail}
          </span>
          <ReadThumb source={picture} name={detail} />
          <Status status={tool.status} />
          <ChevronRight
            aria-hidden
            size={13}
            className={`shrink-0 text-muted transition-transform ${open ? "rotate-90" : ""}`}
          />
        </button>
        {showFile && tool.file && tool.status !== "failed" && (
          <button
            type="button"
            title={t.files.showInPanel}
            aria-label={`${t.files.showInPanel}: ${tool.summary}`}
            onClick={() => showFile(tool.file as string)}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <Files aria-hidden size={14} />
          </button>
        )}
        {showScreen && tool.file && isScreenFile(tool.file) && tool.status !== "failed" && (
          <button
            type="button"
            title={t.screens.showInPanel}
            aria-label={`${t.screens.showInPanel}: ${tool.summary}`}
            onClick={() => showScreen(tool.file as string)}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <LayoutTemplate aria-hidden size={14} />
          </button>
        )}
        {showTerminal && command && (
          <button
            type="button"
            title={t.terminal.showInPanel}
            aria-label={`${t.terminal.showInPanel}: ${detail}`}
            onClick={() => showTerminal()}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <SquareTerminal aria-hidden size={14} />
          </button>
        )}
        {showDesktop && isDesktopTool(tool.name) && (
          <button
            type="button"
            title={t.desktop.panel.showInPanel}
            aria-label={`${t.desktop.panel.showInPanel}: ${title}`}
            onClick={() => showDesktop()}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <Monitor aria-hidden size={14} />
          </button>
        )}
        {showBrowser && isBrowserTool(tool.name) && (
          <button
            type="button"
            title={t.browser.showInPanel}
            aria-label={`${t.browser.showInPanel}: ${title}`}
            onClick={() => showBrowser()}
            className="grid size-6 shrink-0 place-items-center rounded-lg text-muted transition-colors hover:bg-sunken hover:text-ink"
          >
            <Globe aria-hidden size={14} />
          </button>
        )}
      </div>
      {open && (
        <div className="mt-1 mb-2 ml-7 flex animate-rise flex-col gap-1.5" data-selectable>
          <ReadPicture source={picture} name={detail} />
          <p className="text-muted text-xs">
            {command ? t.chat.tools.command : t.chat.tools.input}
          </p>
          <pre className={block}>{command ? command.text : pretty(tool.input)}</pre>
          {command && !command.whole && (
            <p className="text-warn text-xs">{t.chat.tools.commandCut}</p>
          )}
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
          <ToolLine key={item.id} tool={item.body} botId={item.botId} createdAt={item.createdAt} />
        ) : null,
      )}
    </ul>
  );
}
