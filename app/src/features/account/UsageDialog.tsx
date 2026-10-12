// How much of the Claude plan the bots have used (spec 8.1), from the
// usage Claude Code reports while bots work, and the tokens each bot used.

import { useT } from "../../i18n";
import { fromNow } from "../../lib/format";
import type { AgentUsage, UsageWindow } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Callout } from "../../ui/Callout";
import { Dialog } from "../../ui/Dialog";
import { TokensByBot } from "./TokensByBot";

export function UsageDialog({ onClose }: { onClose(): void }) {
  const t = useT();
  const u = t.account.usage;
  const claude = useApp((state) => state.system?.usage ?? null);
  const others = useApp((state) => state.system?.agentUsage ?? NO_USAGE);
  const names = t.bots.dialog.agent.names;
  const named = others.length > 0;
  const shown = [
    ...(claude === null ? [] : [{ agent: "claude" as const, plan: null, usage: claude }]),
    ...others,
  ].filter((entry) => entry.usage.windows.length > 0);

  return (
    <Dialog title={u.title} onClose={onClose}>
      <div className="flex flex-col gap-4 text-sm">
        <p className="text-ink-soft leading-relaxed">{u.intro}</p>
        {shown.length === 0 ? (
          <p className="text-muted">{u.empty}</p>
        ) : (
          shown.map((entry) => {
            const limited =
              entry.usage.status !== "allowed" && entry.usage.status !== "allowed_warning";
            return (
              <section key={entry.agent} className="flex flex-col gap-4">
                {named && (
                  <h3 className="font-medium">
                    {names[entry.agent]}
                    {entry.plan !== null && (
                      <span className="ml-2 font-normal text-muted text-xs">
                        {u.plan(entry.plan)}
                      </span>
                    )}
                  </h3>
                )}
                {limited && (
                  <Callout
                    tone="warn"
                    title={
                      entry.agent === "claude" ? u.limited : u.limitedAgent(names[entry.agent])
                    }
                  />
                )}
                <ul className="flex flex-col gap-4">
                  {entry.usage.windows.map((window) => (
                    <Window key={window.name} window={window} />
                  ))}
                </ul>
                <p className="text-muted text-xs">{u.updated(fromNow(entry.usage.observedAt))}</p>
              </section>
            );
          })
        )}
        <TokensByBot />
      </div>
    </Dialog>
  );
}

const NO_USAGE: AgentUsage[] = [];

function Window({ window }: { window: UsageWindow }) {
  const u = useT().account.usage;
  const percent = Math.round(Math.min(1, Math.max(0, window.utilization)) * 100);
  const name = u.windows[window.name] ?? window.name.replaceAll("_", " ");
  const tone = percent >= 90 ? "bg-danger" : percent >= 70 ? "bg-warn" : "bg-work";
  return (
    <li className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between gap-3">
        <span className="font-medium">{name}</span>
        <span className="text-ink-soft">{u.used(percent)}</span>
      </div>
      <div
        role="progressbar"
        aria-label={name}
        aria-valuenow={percent}
        aria-valuemin={0}
        aria-valuemax={100}
        className="h-2 w-full bg-sunken"
      >
        <div
          className={`h-full origin-left animate-grow ${tone}`}
          style={{ width: `${percent}%` }}
        />
      </div>
      {window.resetsAt !== null && (
        <span className="text-muted text-xs">{u.resets(fromNow(window.resetsAt))}</span>
      )}
    </li>
  );
}
