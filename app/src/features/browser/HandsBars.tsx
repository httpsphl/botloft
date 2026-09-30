// The owner's controls in the browser panel (spec 21.10): taking the
// browser, the bot's request for a hand, and the bar that says the owner
// has it, with the button that gives it back.

import { Check, Hand, Keyboard, MousePointerClick } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import type { Hands } from "./useHands";

/** The bot asked for a hand and the owner has not taken the browser yet. */
export function AskCallout({ bot, task, hands }: { bot: Bot; task: string; hands: Hands }) {
  const t = useT().browser;
  return (
    <div className="mb-3">
      <Callout tone="warn" title={t.help.needs(bot.name)}>
        <p data-selectable>{task}</p>
        <Button
          variant="primary"
          size="sm"
          icon={Hand}
          className="mt-2.5"
          disabled={hands.busy}
          onClick={() => void hands.take()}
        >
          {t.hands.take}
        </Button>
      </Callout>
    </div>
  );
}

/** The owner has the browser. */
export function HeldBar({
  bot,
  task,
  tabs,
  focused,
  hands,
}: {
  bot: Bot;
  task: string | null;
  /** How many tabs are open: with more than one, where the bot goes on. */
  tabs: number;
  focused: boolean;
  hands: Hands;
}) {
  const t = useT().browser;
  const Hint = focused ? Keyboard : MousePointerClick;
  return (
    <div
      role="status"
      className="mb-3 animate-rise rounded-xl border border-accent/45 bg-accent/8 px-3.5 py-3"
    >
      <div className="flex items-start gap-2.5">
        <Hand aria-hidden size={16} className="mt-0.5 shrink-0 text-accent" />
        <div className="min-w-0 flex-1 text-sm">
          <p className="font-semibold">{t.hands.holding}</p>
          <p className="line-clamp-2 text-ink-soft text-xs" data-selectable>
            {task ? t.help.asks(bot.name, task) : t.hands.waits(bot.name)}
          </p>
          {tabs > 1 && <p className="text-muted text-xs">{t.hands.leaves(bot.name)}</p>}
        </div>
      </div>
      <div className="mt-2.5 flex flex-wrap items-center gap-x-3 gap-y-2 pl-6.5">
        <Button
          variant="primary"
          size="sm"
          icon={Check}
          disabled={hands.busy}
          onClick={() => void hands.release()}
        >
          {t.hands.giveBack(bot.name)}
        </Button>
        <span className="flex items-center gap-1.5 text-muted text-xs">
          <Hint aria-hidden size={12} className="shrink-0" />
          {focused ? t.hands.typing : t.hands.clickToType}
        </span>
      </div>
    </div>
  );
}

/** Under the live screen: the owner may take the browser at any time. */
export function TakeBar({ bot, hands }: { bot: Bot; hands: Hands }) {
  const t = useT().browser.hands;
  return (
    <div className="mt-3 flex items-center gap-3">
      <Button
        variant="secondary"
        size="sm"
        icon={Hand}
        disabled={hands.busy}
        onClick={() => void hands.take()}
      >
        {t.take}
      </Button>
      <p className="min-w-0 text-muted text-xs">{t.takeWhy(bot.name)}</p>
    </div>
  );
}
