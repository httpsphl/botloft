// How full the agent's conversation is, and with what (spec 33.3): the
// total, a bar with a color per part, the parts with their share of the
// window, the memory files, and the button to compact now.

import { useId } from "react";
import { useT } from "../../i18n";
import { planPercent, tokens } from "../../lib/format";
import type { Bot, ContextFile, ContextUsage } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { fileAndFolder, freeTokens, liveParts, PART_COLORS } from "./parts";

export function ContextSection({ bot }: { bot: Bot }) {
  const w = useT().overview.context;
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h3 id={id} className="font-semibold text-sm">
        {w.title}
      </h3>
      {bot.context ? (
        <Usage bot={bot} context={bot.context} />
      ) : (
        <p className="text-muted text-sm">{w.unknown(bot.name)}</p>
      )}
    </section>
  );
}

function Usage({ bot, context }: { bot: Bot; context: ContextUsage }) {
  const w = useT().overview.context;
  const window = context.windowTokens;
  const parts = liveParts(context);
  const free = freeTokens(context, parts);
  const percent = planPercent(Math.min(1, context.usedTokens / window));
  return (
    <>
      <div className="flex items-end justify-between gap-3">
        <div>
          <p className="font-semibold text-xl tabular-nums">{w.used(percent)}</p>
          <p className="text-muted text-sm tabular-nums">
            {w.of(tokens(context.usedTokens), tokens(window))}
          </p>
        </div>
        <CompactNow bot={bot} compacting={context.compacting} />
      </div>
      <div
        role="img"
        aria-label={w.bar(percent)}
        className="flex h-2.5 overflow-hidden rounded-full bg-sunken"
      >
        {parts.map((part) => (
          <span
            key={part.kind}
            className="h-full transition-[width] duration-300"
            style={{
              width: `${(part.tokens / window) * 100}%`,
              background: PART_COLORS[part.kind],
            }}
          />
        ))}
      </div>
      {parts.length === 0 ? (
        <p className="text-muted text-sm">{w.noParts}</p>
      ) : (
        <ul aria-label={w.parts} className="flex flex-col gap-1.5">
          {parts.map((part) => (
            <Part
              key={part.kind}
              color={PART_COLORS[part.kind]}
              name={w.kind[part.kind]}
              count={part.tokens}
              window={window}
            />
          ))}
          <Part name={w.free} count={free} window={window} />
        </ul>
      )}
      {context.memoryFiles.length > 0 && <MemoryFiles bot={bot} files={context.memoryFiles} />}
    </>
  );
}

function Part({
  color,
  name,
  count,
  window,
}: {
  /** None for the free space, drawn as an empty square. */
  color?: string;
  name: string;
  count: number;
  window: number;
}) {
  return (
    <li className="flex items-start gap-2.5 text-sm">
      <span
        aria-hidden
        className={`mt-1 size-2.5 shrink-0 rounded-[3px] ${color ? "" : "ring-1 ring-line-strong ring-inset"}`}
        style={color ? { background: color } : undefined}
      />
      <span className="min-w-0 flex-1 break-words text-ink-soft leading-snug">{name}</span>
      <span className="shrink-0 text-muted tabular-nums">{tokens(count)}</span>
      <span className="w-12 shrink-0 text-right font-medium tabular-nums">
        {planPercent(count / window)}
      </span>
    </li>
  );
}

function MemoryFiles({ bot, files }: { bot: Bot; files: ContextFile[] }) {
  const w = useT().overview.context;
  return (
    <div className="flex flex-col gap-1.5">
      <p className="font-medium text-sm">{w.memory}</p>
      <p className="text-muted text-xs">{w.memoryHint(bot.name)}</p>
      <ul aria-label={w.memory} className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1.5">
        {files.map((file) => {
          const { name, folder } = fileAndFolder(file.path);
          return (
            <li
              key={file.path}
              title={file.path}
              className="flex items-baseline gap-2 rounded-lg px-2 py-1 text-sm"
            >
              <span className="shrink-0 font-mono text-xs">{name}</span>
              <span className="min-w-0 flex-1 truncate text-muted text-xs">{folder}</span>
              <span className="shrink-0 text-muted text-xs tabular-nums">
                {tokens(file.tokens)}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

/** The same compaction as the meter under the chat (spec 8.6). */
function CompactNow({ bot, compacting }: { bot: Bot; compacting: boolean }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const m = useT().chat.context;
  // Claude Code and Codex answer a request to compact (spec 30).
  if (bot.agent !== "claude" && bot.agent !== "codex") {
    return null;
  }
  const running = bot.state === "idle" || bot.state === "busy" || bot.state === "needs_approval";
  return (
    <Button
      size="sm"
      disabled={compacting || !running}
      aria-busy={compacting}
      title={running ? m.compactHint(bot.name) : m.notRunning(bot.name)}
      onClick={() =>
        attempt(m.failed, async () => putBot(await api.call("bots.compact", { botId: bot.id })))
      }
    >
      {compacting ? m.compacting : m.compact}
    </Button>
  );
}
