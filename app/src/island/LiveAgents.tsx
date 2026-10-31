// The agents at work, in the island's Home (spec 31.4): the first one big,
// with the steps of its turn and what it does now (an edit's diff, a
// command, the text it writes); the next two in a line each.

import { Check, LoaderCircle, X } from "lucide-react";
import type { CSSProperties } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { commandOf, isCommand } from "../features/chat/command";
import { type DiffLine, editDiffs, isEditTool } from "../features/chat/lineDiff";
import { toolTitle } from "../features/chat/toolNames";
import { useT } from "../i18n";
import type { Bot, BotId, ChatItem, ToolItem } from "../lib/protocol.gen";
import type { Live } from "./useLive";

/** Big and small cards together, before "+N working". */
export const SHOWN = 3;
const STEPS = 4;
const DIFF_LINES = 10;
const TEXT_LINES = 5;

const tools = (items: ChatItem[]): ToolItem[] =>
  items.flatMap((item) => (item.body.kind === "tool" ? [item.body] : []));

function Status({ status }: { status: ToolItem["status"] }) {
  const t = useT();
  if (status === "running") {
    return (
      <LoaderCircle
        aria-label={t.chat.tools.running}
        size={14}
        className="shrink-0 animate-spin text-[var(--island-work)]"
      />
    );
  }
  return status === "done" ? (
    <Check aria-label={t.chat.tools.done} size={14} className="shrink-0 text-[var(--island-ok)]" />
  ) : (
    <X
      aria-label={t.chat.tools.failed}
      size={14}
      className="shrink-0 text-[var(--island-danger)]"
    />
  );
}

const MARK: Record<DiffLine["kind"], string> = { same: " ", added: "+", removed: "−" };

function Diff({ tool, diffs }: { tool: ToolItem; diffs: DiffLine[][] }) {
  const lines = diffs.flat();
  const changed = lines.findIndex((line) => line.kind !== "same");
  const start = Math.max(0, Math.min(changed - 2, lines.length - DIFF_LINES));
  return (
    <div className="island-sunk flex min-w-0 flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-[var(--island-line)] border-b px-3 py-2 text-xs">
        <span className="truncate font-mono">{tool.summary}</span>
        {tool.lines && (
          <span className="shrink-0 font-mono">
            {tool.lines.added > 0 && (
              <span className="text-[var(--island-ok)]">+{tool.lines.added} </span>
            )}
            {tool.lines.removed > 0 && (
              <span className="text-[var(--island-danger)]">−{tool.lines.removed}</span>
            )}
          </span>
        )}
      </div>
      <pre className="py-1.5 font-mono text-[12px] leading-[1.7]">
        {lines.slice(start, start + DIFF_LINES).map((line, i) => (
          <span
            // biome-ignore lint/suspicious/noArrayIndexKey: lines of one fixed text
            key={i}
            data-kind={line.kind}
            className={`flex px-3 ${line.kind === "added" ? "bg-[#4ade80]/12" : line.kind === "removed" ? "bg-[#f87171]/12" : ""}`}
          >
            <span
              aria-hidden={line.kind === "same"}
              className={`w-5 shrink-0 select-none ${line.kind === "added" ? "text-[var(--island-ok)]" : "text-[var(--island-danger)]"}`}
            >
              {MARK[line.kind]}
            </span>
            <span className="min-w-0 truncate text-[var(--island-ink)]">{line.text || " "}</span>
          </span>
        ))}
      </pre>
    </div>
  );
}

/** The last lines of a text, as the agent writes it. */
function Writing({ text }: { text: string }) {
  const lines = text.trimEnd().split("\n").slice(-TEXT_LINES).join("\n");
  return (
    <p className="island-sunk min-w-0 flex-1 whitespace-pre-wrap break-words px-3 py-2.5 text-[13px] text-[var(--island-ink)]">
      {lines}
      <span className="animate-pulse">▍</span>
    </p>
  );
}

/** What the agent does right now, on the right of its card. */
function Now({ live }: { live: Live }) {
  const t = useT();
  const last = live.items.at(-1);
  if (live.writing.trim() !== "") {
    return <Writing text={live.writing} />;
  }
  if (last?.body.kind === "tool") {
    const tool = last.body;
    const diffs = isEditTool(tool.name) ? editDiffs(tool.name, tool.input) : null;
    if (diffs) {
      return <Diff tool={tool} diffs={diffs} />;
    }
    const command = isCommand(tool.name) ? commandOf(tool.input).text : null;
    return (
      <div className="island-sunk flex min-w-0 flex-1 flex-col gap-1.5 px-3 py-2.5 text-[13px]">
        <span className="font-medium">{toolTitle(tool.name, t.tools)}</span>
        {tool.explanation && <span className="island-soft">{tool.explanation}</span>}
        {(command ?? tool.summary) && (
          <code className="line-clamp-4 whitespace-pre-wrap break-all font-mono text-[12.5px] text-[var(--island-ink)]">
            {command ? `❯ ${command}` : tool.summary}
          </code>
        )}
      </div>
    );
  }
  if (last?.body.kind === "reply") {
    return <Writing text={last.body.text} />;
  }
  return <div className="island-sunk min-w-0 flex-1" />;
}

function BigCard({ bot, crew, live }: { bot: Bot; crew: string; live: Live }) {
  const t = useT();
  const steps = tools(live.items).slice(-STEPS);
  return (
    <div className="island-card flex gap-4 p-4">
      <div className="flex w-32 shrink-0 flex-col gap-3">
        <div className="flex flex-col items-center gap-1.5 text-center">
          <span
            className="island-glow island-breathe"
            style={{ "--glow": bot.color } as CSSProperties}
          >
            <BotAvatar color={bot.color} size={56} mood="working" still />
          </span>
          <span className="font-semibold text-[15px]">{bot.name}</span>
          <span className="island-muted text-xs">
            {crew} · {t.island.engines[bot.agent]}
          </span>
        </div>
        <ul className="flex flex-col gap-1.5">
          {steps.map((step) => (
            <li key={step.toolUseId} className="flex items-center gap-2 text-[13px]">
              <Status status={step.status} />
              <span className="truncate">{toolTitle(step.name, t.tools)}</span>
            </li>
          ))}
        </ul>
      </div>
      <Now live={live} />
    </div>
  );
}

function SmallCard({ bot, live }: { bot: Bot; live: Live | undefined }) {
  const t = useT();
  const last = live ? tools(live.items).at(-1) : undefined;
  return (
    <div className="island-card flex min-w-0 items-center gap-3 px-4 py-3">
      <span className="island-breathe inline-grid">
        <BotAvatar color={bot.color} size={32} mood="working" still />
      </span>
      <div className="min-w-0 flex-1 text-[13px]">
        <div className="font-semibold">{bot.name}</div>
        <div className="island-muted truncate">
          {last ? `${toolTitle(last.name, t.tools)} · ${last.summary}` : t.island.thinking}
        </div>
      </div>
      <LoaderCircle
        aria-hidden
        size={14}
        className="shrink-0 animate-spin text-[var(--island-work)]"
      />
    </div>
  );
}

export function LiveAgents({
  working,
  crews,
  live,
}: {
  working: Bot[];
  crews: Map<string, string>;
  live: Map<BotId, Live>;
}) {
  const t = useT();
  const [first, ...rest] = working;
  if (!first) {
    return null;
  }
  const more = working.length - SHOWN;
  return (
    <>
      <BigCard
        bot={first}
        crew={crews.get(first.crewId) ?? ""}
        live={live.get(first.id) ?? { items: [], writing: "" }}
      />
      {rest.length > 0 && (
        <div className="grid grid-cols-2 gap-2">
          {rest.slice(0, SHOWN - 1).map((bot) => (
            <SmallCard key={bot.id} bot={bot} live={live.get(bot.id)} />
          ))}
        </div>
      )}
      {more > 0 && <p className="island-muted px-2 text-xs">{t.island.moreWorking(more)}</p>}
    </>
  );
}
