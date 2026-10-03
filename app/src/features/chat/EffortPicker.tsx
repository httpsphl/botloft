// How much the bot thinks before it answers (spec 7.4), on a slider below
// the chat as in Claude's apps: faster on the left, smarter on the right,
// with the level its model uses by itself marked as the recommended one.

import { ChevronDown, Gauge } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import { modelName } from "../../lib/models";
import type { Bot, BotEffort } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { POPOVER } from "../../ui/surface";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";

const LEVELS = ["low", "medium", "high", "xhigh", "max"] as const;
type Level = (typeof LEVELS)[number];
const LAST = LEVELS.length - 1;
/** Where the "Recommended" word sits under the first and the last stop. */
const EDGE: Record<number, string> = { 0: "-4px", [LAST]: "calc(-100% + 4px)" };
/** How long the slider rests on a stop before the bot is told. */
const SETTLE_MS = 400;

const isLevel = (value: string | null): value is Level =>
  value !== null && (LEVELS as readonly string[]).includes(value);

export function EffortPicker({ bot, onLater }: { bot: Bot; onLater(text: string): void }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const m = useT().chat.effort;
  const [open, setOpen] = useState(false);
  /** Where the owner left the slider, until the bot has it. */
  const [draft, setDraft] = useState<Level | null>(null);
  const root = useRef<HTMLDivElement>(null);
  const settle = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  /** The level the bot's model uses by itself, once Claude Code said it. */
  const recommended = isLevel(bot.effortDefault) ? bot.effortDefault : null;
  const unavailable = bot.effortDefault === "none";
  const chosen = bot.effort === "default" ? recommended : bot.effort;
  const shown = draft ?? chosen;
  const label = unavailable ? m.unavailable : shown ? m.names[shown] : m.recommended;

  const set = async (effort: BotEffort, name: string) => {
    if (effort !== bot.effort) {
      // The bot restarts on the new level once nothing is in progress.
      const working = bot.state === "busy" || bot.state === "needs_approval";
      await attempt(m.failed, async () => {
        putBot(await api.call("bots.setEffort", { botId: bot.id, effort }));
        if (working) {
          onLater(m.later(bot.name, name));
        }
      });
    }
    setDraft(null);
  };
  // The recommended stop follows the model: it is the bot's default.
  const commit = (level: Level) => set(level === recommended ? "default" : level, m.names[level]);
  const latest = useRef(commit);
  latest.current = commit;
  const pending = useRef<Level | null>(null);

  const flush = useCallback(() => {
    clearTimeout(settle.current);
    const level = pending.current;
    pending.current = null;
    if (level) {
      void latest.current(level);
    }
  }, []);
  // A stop the owner left the slider on is not lost with the menu.
  useEffect(() => flush, [flush]);

  const close = useCallback(() => {
    flush();
    setOpen(false);
  }, [flush]);
  useDismiss(open, root, close);

  const slide = (index: number) => {
    const level = LEVELS[index];
    if (!level) {
      return;
    }
    setDraft(level);
    pending.current = level;
    clearTimeout(settle.current);
    settle.current = setTimeout(flush, SETTLE_MS);
  };

  const model = bot.modelInUse ? modelName(bot.modelInUse) : null;
  const at = (level: Level) => LEVELS.indexOf(level);

  return (
    <div ref={root} className="shrink-0">
      {open && (
        <div
          role="dialog"
          aria-label={m.title}
          className={`absolute right-2.5 bottom-full mb-2 w-[22rem] max-w-[calc(100%-1.25rem)] origin-bottom-right ${POPOVER} px-3.5 pt-3 pb-3`}
        >
          <p className="font-medium text-muted text-xs">{m.title}</p>
          {unavailable ? (
            <p className="mt-2 text-ink-soft text-sm">{m.none(bot.name, model ?? m.thisModel)}</p>
          ) : (
            <>
              <div className="mt-2.5 flex justify-between text-muted text-xs">
                <span>{m.faster}</span>
                <span>{m.smarter}</span>
              </div>
              <input
                type="range"
                min={0}
                max={LAST}
                step={1}
                value={shown ? at(shown) : LAST / 2}
                data-unset={shown ? undefined : ""}
                aria-label={m.title}
                aria-valuetext={
                  shown
                    ? `${m.names[shown]}${shown === recommended ? ` (${m.recommended})` : ""}`
                    : m.recommended
                }
                onChange={(event) => slide(Number(event.target.value))}
                className="effort-slider mt-1"
              />
              {/* The stops sit under where the thumb's center goes. */}
              <div aria-hidden className="relative mx-2 h-7">
                {LEVELS.map((level, index) => (
                  <span
                    key={level}
                    style={{ left: `${(index / LAST) * 100}%` }}
                    className={`absolute top-0 h-1.5 w-1.5 -translate-x-1/2 rounded-full ${
                      level === recommended ? "bg-accent" : "bg-line-strong"
                    }`}
                  />
                ))}
                {recommended && (
                  <span
                    style={{
                      left: `${(at(recommended) / LAST) * 100}%`,
                      transform: `translateX(${EDGE[at(recommended)] ?? "-50%"})`,
                    }}
                    className="absolute top-2.5 whitespace-nowrap text-[0.6875rem] text-accent-text"
                  >
                    {m.recommended}
                  </span>
                )}
              </div>
              <p className="font-medium text-ink text-sm">
                {shown ? m.names[shown] : m.recommended}
                {shown && shown === recommended && (
                  <span className="font-normal text-muted">
                    {" · "}
                    {model ? m.recommendedFor(model) : m.recommended}
                  </span>
                )}
              </p>
              <p className="text-muted text-xs">
                {shown ? m.hints[shown](bot.name) : m.unknown(bot.name)}
              </p>
            </>
          )}
          <div className="mt-2.5 flex items-center gap-3 border-line border-t pt-2.5">
            <p className="min-w-0 flex-1 text-muted text-xs">{m.cost}</p>
            {bot.effort !== "default" && !unavailable && (
              <button
                type="button"
                onClick={() => {
                  clearTimeout(settle.current);
                  pending.current = null;
                  void set("default", recommended ? m.names[recommended] : m.recommended);
                }}
                className="shrink-0 rounded-lg px-1.5 py-0.5 font-medium text-ink-soft text-xs hover:bg-sunken hover:text-ink"
              >
                {m.useRecommended}
              </button>
            )}
          </div>
        </div>
      )}
      <button
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-label={m.button(label)}
        title={
          unavailable
            ? m.none(bot.name, model ?? m.thisModel)
            : shown
              ? m.hints[shown](bot.name)
              : m.unknown(bot.name)
        }
        onClick={() => (open ? close() : setOpen(true))}
        className={`flex h-8 max-w-full items-center gap-1.5 rounded-full px-2.5 text-sm transition-colors hover:bg-sunken hover:text-ink ${
          open ? "bg-sunken text-ink" : unavailable ? "text-muted" : "text-ink-soft"
        }`}
      >
        <Gauge aria-hidden size={15} className="shrink-0" />
        <span className="hidden truncate @md:inline">{shown ? m.names[shown] : m.short}</span>
        <ChevronDown
          aria-hidden
          size={14}
          className={`hidden shrink-0 opacity-60 transition-transform duration-200 @md:block ${open ? "rotate-180" : ""}`}
        />
      </button>
    </div>
  );
}
