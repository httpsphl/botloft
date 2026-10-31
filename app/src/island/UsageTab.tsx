// The island's Usage tab (spec 31.4): the windows of the Claude plan, the
// other engines' when there are any, and the next three routines of every
// crew, each with "Run now".

import { type CSSProperties, useEffect, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import { fromNow } from "../lib/format";
import type { AgentUsage, Bot, Routine, UsageWindow } from "../lib/protocol.gen";

/** From here on, a window shows in the warning color (spec 31.4). */
export const WARN_AT = 0.8;
/** How often the island reads the plan again while it shows. */
const EVERY_MS = 60_000;
const ROUTINES = 3;

export interface Usage {
  claude: UsageWindow[];
  others: AgentUsage[];
  routines: Routine[];
}

/** The plan's windows and the next routines, read again every minute and
 *  whenever `version` changes. */
export function useUsage(api: BotloftApi, version: number): Usage | null {
  const [usage, setUsage] = useState<Usage | null>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `version` asks to read again
  useEffect(() => {
    let alive = true;
    const read = () =>
      Promise.all([api.call("system.status"), api.call("routines.list", {})]).then(
        ([status, routines]) => {
          if (!alive) {
            return;
          }
          const next = routines
            .filter(
              (routine) =>
                routine.enabled && routine.archivedAt === null && routine.nextRunAt !== null,
            )
            .sort((a, b) => (a.nextRunAt ?? 0) - (b.nextRunAt ?? 0))
            .slice(0, ROUTINES);
          setUsage({
            claude: status.usage?.windows ?? [],
            others: status.agentUsage,
            routines: next,
          });
        },
        () => {},
      );
    read();
    const timer = setInterval(read, EVERY_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [api, version]);
  return usage;
}

/** The 5-hour window's share, for the ring on the pill. */
export function fiveHour(usage: Usage | null): number {
  return usage?.claude.find((window) => window.name === "five_hour")?.utilization ?? 0;
}

function Bar({ window }: { window: UsageWindow }) {
  const t = useT();
  const words = t.account.usage;
  const percent = Math.round(window.utilization * 100);
  const warn = window.utilization >= WARN_AT;
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex justify-between gap-3 text-[13px]">
        <span className="font-medium">{words.windows[window.name] ?? window.name}</span>
        <span className={warn ? "text-[var(--island-warn)]" : "island-muted"}>
          {words.used(percent)}
          {window.resetsAt !== null && ` · ${words.resets(fromNow(window.resetsAt))}`}
        </span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-[var(--island-line)]">
        <div
          data-warn={warn}
          className={`h-full rounded-full ${warn ? "bg-[var(--island-warn)]" : "bg-white"}`}
          style={{ width: `${Math.min(100, percent)}%` }}
        />
      </div>
    </div>
  );
}

export function UsageTab({
  api,
  usage,
  bots,
}: {
  api: BotloftApi;
  usage: Usage | null;
  bots: Bot[];
}) {
  const t = useT();
  const [ran, setRan] = useState<string | null>(null);
  if (!usage) {
    return null;
  }
  const run = (routine: Routine) =>
    api.call("routines.runNow", { routineId: routine.id }).then(
      () => setRan(routine.id),
      () => {},
    );
  return (
    <div className="grid grid-cols-2 gap-2">
      <div className="island-card flex flex-col gap-4 p-4">
        <span className="font-semibold text-[15px]">{t.island.usage.plan}</span>
        {usage.claude.length === 0 && (
          <p className="island-muted text-[13px]">{t.account.usage.empty}</p>
        )}
        {usage.claude.map((window) => (
          <Bar key={window.name} window={window} />
        ))}
        {usage.others.map((other) => (
          <div key={other.agent} className="flex flex-col gap-3">
            <span className="island-muted text-xs">{t.island.engines[other.agent]}</span>
            {other.usage.windows.map((window) => (
              <Bar key={window.name} window={window} />
            ))}
          </div>
        ))}
      </div>
      <div className="island-card flex flex-col gap-2.5 p-4">
        <span className="font-semibold text-[15px]">{t.island.usage.routines}</span>
        {usage.routines.length === 0 && (
          <p className="island-muted text-[13px]">{t.island.usage.noRoutines}</p>
        )}
        {usage.routines.map((routine) => {
          const bot = bots.find((have) => have.id === routine.botId);
          return (
            <div key={routine.id} className="flex items-center gap-2.5 text-[13px]">
              {bot && (
                <span className="inline-grid" style={{ "--glow": bot.color } as CSSProperties}>
                  <BotAvatar color={bot.color} size={22} mood="idle" still />
                </span>
              )}
              <span
                className="min-w-0 flex-1 truncate"
                title={bot ? `${bot.name} · ${routine.name}` : routine.name}
              >
                {routine.name}
              </span>
              <span className="island-muted shrink-0">
                {fromNow(routine.nextRunAt ?? Date.now())}
              </span>
              <button
                type="button"
                className="island-button shrink-0 px-2.5 py-0.5 text-xs"
                data-kind="option"
                disabled={ran === routine.id}
                onClick={() => run(routine)}
              >
                {ran === routine.id ? t.island.usage.started : t.island.usage.runNow}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
