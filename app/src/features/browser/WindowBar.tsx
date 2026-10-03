// The bot's browser in a window of its own (spec 21.11): the button that
// opens it, for sites that refuse to sign in to a browser a program drives,
// and the bar that says it is open while the bot waits.

import { AppWindow } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import type { Hands } from "./useHands";

/** Opens the bot's browser in a window of its own. */
export function WindowButton({ hands }: { hands: Hands }) {
  const t = useT().browser.window;
  return (
    <Button
      variant="secondary"
      size="sm"
      icon={AppWindow}
      disabled={hands.busy}
      onClick={() => void hands.window()}
    >
      {t.open}
    </Button>
  );
}

/** The browser is open in a window of its own. */
export function WindowBar({ bot }: { bot: Bot }) {
  const t = useT().browser.window;
  return (
    <div
      role="status"
      className="mb-3 animate-rise rounded-xl border border-accent/45 bg-accent/8 px-3.5 py-3"
    >
      <div className="flex items-start gap-2.5">
        <AppWindow aria-hidden size={16} className="mt-0.5 shrink-0 text-accent" />
        <div className="min-w-0 flex-1 text-sm">
          <p className="font-semibold">{t.title}</p>
          <p className="text-ink-soft text-xs">{t.body(bot.name)}</p>
        </div>
      </div>
    </div>
  );
}
