// How much of the Claude plan the bots have used (spec 8.1), from the
// usage Claude Code reports while bots work.

import { useT } from "../../i18n";
import { fromNow } from "../../lib/format";
import type { UsageWindow } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Callout } from "../../ui/Callout";
import { Dialog } from "../../ui/Dialog";

export function UsageDialog({ onClose }: { onClose(): void }) {
  const u = useT().account.usage;
  const usage = useApp((state) => state.system?.usage ?? null);
  const limited =
    usage !== null && usage.status !== "allowed" && usage.status !== "allowed_warning";

  return (
    <Dialog title={u.title} onClose={onClose}>
      <div className="flex flex-col gap-4 text-sm">
        <p className="text-ink-soft leading-relaxed">{u.intro}</p>
        {usage === null || usage.windows.length === 0 ? (
          <p className="text-muted">{u.empty}</p>
        ) : (
          <>
            {limited && <Callout tone="warn" title={u.limited} />}
            <ul className="flex flex-col gap-4">
              {usage.windows.map((window) => (
                <Window key={window.name} window={window} />
              ))}
            </ul>
            <p className="text-muted text-xs">{u.updated(fromNow(usage.observedAt))}</p>
          </>
        )}
      </div>
    </Dialog>
  );
}

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
        <div className={`h-full ${tone}`} style={{ width: `${percent}%` }} />
      </div>
      {window.resetsAt !== null && (
        <span className="text-muted text-xs">{u.resets(fromNow(window.resetsAt))}</span>
      )}
    </li>
  );
}
