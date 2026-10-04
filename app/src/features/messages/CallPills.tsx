// Bots calling each other, live (spec 15.3): one pill per call, with the
// mascots of both bots stacked and "Calling Writer…" until the other bot
// picks the message up, then "Writer picked it up" for a moment.

import { Check } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useT } from "../../i18n";
import { type Call, type LiveCall, liveCalls, nextChange } from "../../store/calls";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";

/** The calls `keep` wants, kept current as they ring and end. */
export function useLiveCalls(keep: (call: Call) => boolean, key: string): LiveCall[] {
  const calls = useApp((state) => state.calls);
  const deliveries = useApp((state) => state.deliveries);
  const [now, setNow] = useState(Date.now);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `key` names what `keep` keeps
  const shown = useMemo(
    () => liveCalls({ calls, deliveries }, Math.max(now, Date.now()), keep),
    [calls, deliveries, now, key],
  );
  // Look again when the next one ends by itself.
  useEffect(() => {
    const wait = nextChange(shown, Date.now());
    if (wait === null) return;
    const timer = setTimeout(() => setNow(Date.now()), wait);
    return () => clearTimeout(timer);
  }, [shown]);
  return shown;
}

function Pill({ call }: { call: LiveCall }) {
  const t = useT().messages.calls;
  const from = useApp((state) => state.bots[call.fromBotId]);
  const to = useApp((state) => state.bots[call.toBotId]);
  if (!from || !to) {
    return null;
  }
  const picked = call.readAt !== null;
  return (
    <li
      className={`call-pill flex animate-rise items-center gap-2 rounded-full border bg-panel py-1 pr-3 pl-1 text-sm shadow-sm ${
        picked ? "border-ok/45" : "border-line"
      }`}
      data-picked={picked || undefined}
    >
      <span aria-hidden className="flex items-center">
        <span className="call-avatar">
          <BotAvatar botId={from.id} color={from.color} size={20} />
        </span>
        <span className="call-avatar -ml-2">
          <BotAvatar botId={to.id} color={to.color} size={20} />
        </span>
      </span>
      {picked ? (
        <span className="flex items-center gap-1 text-ok">
          <Check aria-hidden size={13} className="shrink-0" />
          <span className="sr-only">{t.caller(from.name)}</span>
          {t.picked(to.name)}
        </span>
      ) : (
        <span className="flex items-center gap-1.5 text-ink-soft">
          {/* The mascots say who calls; a screen reader hears it. */}
          <span className="sr-only">{t.caller(from.name)}</span>
          <span className="truncate">{t.calling(to.name)}</span>
          <span aria-hidden className="flex items-center gap-0.5">
            {[0, 160, 320].map((delay) => (
              <span
                key={delay}
                className="size-1 animate-dot rounded-full bg-accent"
                style={{ animationDelay: `${delay}ms` }}
              />
            ))}
          </span>
        </span>
      )}
    </li>
  );
}

/** The calls going on, as pills; nothing when there are none. */
export function CallPills({ calls, className = "" }: { calls: LiveCall[]; className?: string }) {
  const t = useT().messages.calls;
  if (calls.length === 0) {
    return null;
  }
  return (
    <ul aria-label={t.label} aria-live="polite" className={`flex flex-wrap gap-2 ${className}`}>
      {calls.map((call) => (
        <Pill key={call.messageId} call={call} />
      ))}
    </ul>
  );
}
