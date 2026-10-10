// How full the bot's conversation is (spec 8.6): a ring below the chat
// that opens the numbers, how far it is from compacting by itself, and
// the button to compact it now.

import { useCallback, useRef, useState } from "react";
import { useT } from "../../i18n";
import { tokens } from "../../lib/format";
import type { Bot } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { POPOVER } from "../../ui/surface";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";

const RADIUS = 7;
const AROUND = 2 * Math.PI * RADIUS;
/** From this share of the way to compacting by itself, the ring warns. */
const NEAR = 0.8;

export function ContextMeter({ bot, onLater }: { bot: Bot; onLater(text: string): void }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const m = useT().chat.context;
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);

  const context = bot.context;
  if (!context) {
    return null;
  }
  const { usedTokens, windowTokens, autoCompactTokens, compacting } = context;
  const share = Math.min(1, usedTokens / windowTokens);
  const percent = Math.round(share * 100);
  const used = tokens(usedTokens);
  const total = tokens(windowTokens);
  const limit = autoCompactTokens ?? windowTokens;
  const near = usedTokens >= limit * NEAR;
  const left = autoCompactTokens === null ? null : autoCompactTokens - usedTokens;
  const claude = bot.agent === "claude";
  // Claude Code and Codex answer a request to compact (spec 30).
  const compactable = claude || bot.agent === "codex";
  const running = bot.state === "idle" || bot.state === "busy" || bot.state === "needs_approval";

  const compact = async () => {
    // It waits its turn behind what the bot is doing.
    const working = bot.state === "busy" || bot.state === "needs_approval";
    await attempt(m.failed, async () => {
      putBot(await api.call("bots.compact", { botId: bot.id }));
      if (working) {
        onLater(m.later(bot.name));
      }
    });
  };

  return (
    <div ref={root} className="shrink-0">
      {open && (
        <div
          role="dialog"
          aria-label={m.title}
          className={`absolute right-2.5 bottom-full mb-2 w-[21rem] max-w-[calc(100%-1.25rem)] origin-bottom-right ${POPOVER} px-3.5 pt-3 pb-3.5`}
        >
          <p className="font-medium text-muted text-xs">{m.title}</p>
          <p className="mt-1.5 font-semibold text-base text-ink tabular-nums">
            {m.used(used, total, percent)}
          </p>
          <div aria-hidden className="relative mt-2 h-1.5 rounded-full bg-sunken">
            <div
              className={`h-full rounded-full transition-[width] duration-300 ${near ? "bg-warn" : "bg-ink-soft"}`}
              style={{ width: `${Math.max(share * 100, 1.5)}%` }}
            />
            {autoCompactTokens !== null && (
              <span
                className="absolute -top-0.5 h-2.5 w-px bg-muted"
                style={{ left: `${Math.min(100, (autoCompactTokens / windowTokens) * 100)}%` }}
              />
            )}
          </div>
          <p className="mt-2 text-ink-soft text-sm">
            {!compactable
              ? m.estimate
              : bot.agent === "codex"
                ? m.byItself
                : left === null
                  ? m.noAuto
                  : left > 0
                    ? m.autoLeft(tokens(left), tokens(autoCompactTokens ?? 0))
                    : m.autoNow}
          </p>
          <p className="mt-1.5 text-muted text-xs">{m.about(bot.name)}</p>
          {/* Only Claude Code answers a request to compact (spec 30). */}
          {compactable && (
            <div className="mt-3 border-line border-t pt-3">
              <Button
                size="sm"
                disabled={compacting || !running}
                aria-busy={compacting}
                onClick={compact}
              >
                {compacting ? m.compacting : m.compact}
              </Button>
              <p className="mt-1.5 text-muted text-xs">
                {running || compacting ? m.compactHint(bot.name) : m.notRunning(bot.name)}
              </p>
            </div>
          )}
        </div>
      )}
      <button
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-label={m.button(used, total, percent)}
        title={m.used(used, total, percent)}
        onClick={() => setOpen(!open)}
        className={`grid h-8 w-8 place-items-center rounded-full transition-colors hover:bg-sunken ${
          open ? "bg-sunken" : ""
        } ${near ? "text-warn" : "text-ink-soft hover:text-ink"}`}
      >
        <svg
          aria-hidden
          viewBox="0 0 20 20"
          className={`h-[18px] w-[18px] -rotate-90 ${compacting ? "animate-pulse" : ""}`}
        >
          <circle
            cx="10"
            cy="10"
            r={RADIUS}
            fill="none"
            strokeWidth="2.5"
            className="stroke-line-strong"
          />
          <circle
            cx="10"
            cy="10"
            r={RADIUS}
            fill="none"
            strokeWidth="2.5"
            strokeLinecap="round"
            stroke="currentColor"
            strokeDasharray={`${Math.max(share, 0.02) * AROUND} ${AROUND}`}
            className="transition-[stroke-dasharray] duration-300"
          />
        </svg>
      </button>
    </div>
  );
}
