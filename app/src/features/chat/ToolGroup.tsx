// Tool calls that ran together, folded into one line (spec 15.3). While
// the bot works, the line says what it is doing now and changes with each
// call; once it is done, the line sums the group up. The arrow opens every
// call, each with its own line.

import { ChevronRight, LoaderCircle } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { ToolLines } from "./ToolLines";
import { toolDetail, toolTitle } from "./toolNames";
import { toolSummary } from "./toolSummary";

/** The call the bot is on, swapped in as the next one starts. */
function Now({ tool }: { tool: ToolItem }) {
  const t = useT();
  const detail = tool.explanation || toolDetail(tool, t.tools);
  return (
    <span className="flex min-w-0 animate-rise items-center gap-2">
      <LoaderCircle
        aria-label={t.chat.tools.running}
        size={14}
        className="shrink-0 animate-spin text-work"
      />
      <span className="activity-shine shrink-0 font-medium">{toolTitle(tool.name, t.tools)}</span>
      {detail && (
        <span
          className={`min-w-0 truncate text-muted text-xs ${tool.explanation ? "" : "font-mono"}`}
          title={detail}
        >
          {detail}
        </span>
      )}
    </span>
  );
}

export function ToolGroup({ items, active }: { items: ChatItem[]; active: boolean }) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const calls = items.flatMap((item) =>
    item.body.kind === "tool" ? [{ id: item.id, tool: item.body }] : [],
  );
  const now = calls.at(-1);
  // One finished call is already a single line.
  if (!now || (!active && calls.length === 1)) {
    return <ToolLines items={items} />;
  }
  return (
    <div className="-mx-1.5 flex flex-col">
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="flex min-w-0 max-w-full items-center gap-2 self-start rounded-lg px-1.5 py-1 text-left text-muted text-sm transition-colors hover:bg-sunken hover:text-ink"
      >
        {active ? (
          <Now key={now.id} tool={now.tool} />
        ) : (
          <span className="min-w-0 truncate">
            {toolSummary(
              calls.map((call) => call.tool),
              t.chat.tools.group,
            )}
          </span>
        )}
        <ChevronRight
          aria-hidden
          size={13}
          className={`shrink-0 transition-transform ${open ? "rotate-90" : ""}`}
        />
      </button>
      {open && (
        <div className="ml-3 animate-rise border-line border-l pl-3">
          <ToolLines items={items} />
        </div>
      )}
    </div>
  );
}
