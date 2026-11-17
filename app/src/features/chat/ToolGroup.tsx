// Tool calls that ran together, folded into one line (spec 15.3). While
// the bot works, the line says what it is doing now and changes with each
// call; once it is done, the line sums the group up. The arrow opens every
// call, each with its own line. A call made again and again gets a mark
// beside the line (repeats.ts).

import { ChevronRight, LoaderCircle, Repeat2 } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ChatItem, ToolItem } from "../../lib/protocol.gen";
import { TONES } from "../../ui/tone";
import { ReadThumb, useReadImage } from "./readImage";
import type { Repeat } from "./repeats";
import { ToolLines } from "./ToolLines";
import { toolDetail, toolTitle } from "./toolNames";
import { toolSummary } from "./toolSummary";

/** The call the bot is on, swapped in as the next one starts. */
function Now({ tool, botId }: { tool: ToolItem; botId: string }) {
  const t = useT();
  const detail = tool.explanation || toolDetail(tool, t.tools);
  const picture = useReadImage(botId, tool);
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
      <ReadThumb source={picture} name={detail} />
    </span>
  );
}

/** "Same call 5×", with what the call was and why it matters on hover. */
function RepeatMark({ repeat, bot }: { repeat: Repeat; bot: string }) {
  const t = useT();
  const { tool } = repeat;
  const what = [toolTitle(tool.name, t.tools), tool.explanation || toolDetail(tool, t.tools)]
    .filter(Boolean)
    .join(": ");
  return (
    <span
      title={t.chat.tools.repeatHint(bot, repeat.count, what)}
      className={`flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 font-medium text-xs ${TONES.warn.soft}`}
    >
      <Repeat2 aria-hidden size={12} />
      {t.chat.tools.repeat(repeat.count)}
    </span>
  );
}

export function ToolGroup({
  items,
  active,
  repeat = null,
  bot = "",
}: {
  items: ChatItem[];
  active: boolean;
  /** The call of this group the bot keeps making, if any (repeats.ts). */
  repeat?: Repeat | null;
  bot?: string;
}) {
  const t = useT();
  const [open, setOpen] = useState(false);
  const calls = items.flatMap((item) =>
    item.body.kind === "tool" ? [{ id: item.id, botId: item.botId, tool: item.body }] : [],
  );
  const now = calls.at(-1);
  // One finished call is already a single line, unless it needs its mark.
  if (!now || (!active && calls.length === 1 && !repeat)) {
    return <ToolLines items={items} />;
  }
  return (
    <div className="-mx-1.5 flex flex-col">
      <div className="flex min-w-0 max-w-full items-center gap-1.5 self-start">
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="flex min-w-0 max-w-full items-center gap-2 rounded-lg px-1.5 py-1 text-left text-muted text-sm transition-colors hover:bg-sunken hover:text-ink"
        >
          {active ? (
            <Now key={now.id} tool={now.tool} botId={now.botId} />
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
        {repeat && <RepeatMark repeat={repeat} bot={bot} />}
      </div>
      {open && (
        <div className="ml-3 animate-rise border-line border-l pl-3">
          <ToolLines items={items} />
        </div>
      )}
    </div>
  );
}
