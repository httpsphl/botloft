// What the history dialogs show while the daemon reads or summarizes
// (spec 36.7): reading a big history takes from seconds to a minute, so the
// Maker works on screen, the step is said in words and the seconds count,
// and nobody takes the wait for a frozen window.

import { RotateCw } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";
import { BotAvatar, MAKER_COLOR } from "../bots/BotAvatar";

export function ImportWorking({ title, hint }: { title: string; hint: string }) {
  const h = useT().maker.history;
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    const started = Date.now();
    const timer = setInterval(() => setSeconds(Math.floor((Date.now() - started) / 1000)), 1000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div
      role="status"
      className="flex items-center gap-4 rounded-xl border border-line bg-sunken px-4 py-4"
    >
      <BotAvatar color={MAKER_COLOR} size={72} mood="working" maker />
      <div className="flex min-w-0 flex-col gap-1">
        <p className="font-medium text-ink">{title}</p>
        <p className="text-ink-soft text-xs leading-relaxed">{hint}</p>
        <p className="text-muted text-xs tabular-nums">{h.elapsed(seconds)}</p>
      </div>
    </div>
  );
}

/** A reading that failed, with a way to try again. */
export function ImportFailed({ text, onRetry }: { text: string; onRetry(): void }) {
  const h = useT().maker.history;
  return (
    <div
      role="alert"
      className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-line px-4 py-3"
    >
      <p className="text-ink-soft">{text}</p>
      <Button size="sm" icon={RotateCw} onClick={onRetry}>
        {h.retry}
      </Button>
    </div>
  );
}
