// What bots did on the owner's desktop while the owner was away (spec
// 24.8): one line per bot and app above the main pane, each opening the
// bot's chat where it began, until the owner dismisses them.

import { MessageSquare, Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { DesktopAwayUse } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";

/** The list now, kept current by `desktop.away`. */
function useAwayUses(): DesktopAwayUse[] {
  const api = useApi();
  const [uses, setUses] = useState<DesktopAwayUse[]>([]);
  useEffect(() => {
    let alive = true;
    let heard = false;
    const stop = api.subscribe((event) => {
      if (event.name === "desktop.away") {
        heard = true;
        setUses(event.params);
      }
    });
    api.call("desktop.awayUses").then(
      (listed) => alive && !heard && setUses(listed),
      // An older daemon has none.
      () => undefined,
    );
    return () => {
      alive = false;
      stop();
    };
  }, [api]);
  return uses;
}

export function AwayNotice() {
  const words = useT().desktop.away;
  const api = useApi();
  const uses = useAwayUses();
  const bots = useApp((state) => state.bots);
  const openAt = useApp((state) => state.openAt);
  const selectBot = useApp((state) => state.selectBot);
  const shown = uses.filter((used) => bots[used.botId]);
  if (shown.length === 0) {
    return null;
  }
  return (
    <section
      aria-label={words.label}
      className="flex items-start gap-3 border-line border-b bg-sunken px-4 py-2.5"
    >
      <Monitor aria-hidden size={16} className="mt-1 shrink-0 text-muted" />
      <ul className="flex min-w-0 flex-1 flex-col gap-1">
        {shown.map((used) => (
          <li key={`${used.botId} ${used.app}`} className="flex items-center gap-2 text-sm">
            <span className="min-w-0 flex-1">
              {words.used(bots[used.botId]?.name ?? "", used.app)}
            </span>
            <Button
              variant="ghost"
              size="sm"
              icon={MessageSquare}
              onClick={() =>
                used.itemId ? openAt(used.botId, used.itemId) : selectBot(used.botId)
              }
            >
              {words.open}
            </Button>
          </li>
        ))}
      </ul>
      <Button
        variant="secondary"
        size="sm"
        onClick={() => attempt(words.dismissFailed, () => api.call("desktop.dismissAway"))}
      >
        {words.dismiss}
      </Button>
    </section>
  );
}
