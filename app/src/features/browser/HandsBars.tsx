// The owner's controls in the browser panel (spec 21.10): the bot's request
// for a hand, and the pill under the live screen that says who is in
// control, with the button that takes the browser or gives it back. Under
// it, what the owner should know, and a window of its own for sites that
// refuse to sign in here (spec 21.11).

import { Check, ChevronDown, Hand, Keyboard, MousePointerClick } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import type { Hands } from "./useHands";
import { WindowButton } from "./WindowBar";

/** The bot asked for a hand and the owner has not taken the browser yet. */
export function AskCallout({ bot, task, hands }: { bot: Bot; task: string; hands: Hands }) {
  const t = useT().browser;
  return (
    <div className="mb-3">
      <Callout tone="warn" title={t.help.needs(bot.name)}>
        <p data-selectable>{task}</p>
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button
            variant="primary"
            size="sm"
            icon={Hand}
            disabled={hands.busy}
            onClick={() => void hands.take()}
          >
            {t.hands.take}
          </Button>
          <WindowButton hands={hands} />
        </div>
        <p className="mt-2 text-muted text-xs">{t.window.hint}</p>
      </Callout>
    </div>
  );
}

/**
 * Who is in control, hanging from the bottom of the live screen: the bot,
 * with the button to take it, or the owner, with the one to give it back.
 */
export function ControlPill({ bot, hands }: { bot: Bot; hands: Hands }) {
  const t = useT().browser.hands;
  const held = hands.held;
  return (
    <div
      role="status"
      className={`control-pill relative z-10 mx-auto -mt-5 flex max-w-full items-center gap-2 rounded-full border bg-panel py-1 pr-1 pl-2 shadow-lift ${
        held ? "border-accent" : "border-line"
      }`}
    >
      {held ? (
        <Hand aria-hidden size={16} className="ml-1 shrink-0 text-accent" />
      ) : (
        <BotAvatar color={bot.color} size={22} mood={moodOf(bot)} />
      )}
      <span className="min-w-0 shrink-[2] truncate font-medium text-sm @max-xs:sr-only">
        {held ? t.holding : t.botHas(bot.name)}
      </span>
      <Button
        variant={held ? "primary" : "secondary"}
        size="sm"
        icon={held ? Check : Hand}
        disabled={hands.busy}
        onClick={() => void (held ? hands.release() : hands.take())}
        className="min-w-0 shrink! rounded-full! px-3!"
      >
        <span className="truncate">{held ? t.giveBack(bot.name) : t.take}</span>
      </Button>
    </div>
  );
}

/**
 * The window button, and why to take the browser or use a window, folded:
 * an owner who knows it does not read it every time.
 */
function WindowAndWhy({ hands, why }: { hands: Hands; why: string }) {
  const t = useT().browser.hands;
  const [open, setOpen] = useState(false);
  return (
    <>
      <div className="flex flex-wrap items-center justify-center gap-x-2 gap-y-1">
        <WindowButton hands={hands} />
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="flex h-7 items-center gap-1 rounded-lg px-2 text-muted text-xs transition-colors hover:bg-sunken hover:text-ink"
        >
          {t.howItWorks}
          <ChevronDown
            aria-hidden
            size={12}
            className={`transition-transform ${open ? "rotate-180" : ""}`}
          />
        </button>
      </div>
      {open && <p className="max-w-md animate-rise text-muted text-xs">{why}</p>}
    </>
  );
}

/** Under the pill, while the bot has the browser: how to take it. */
export function TakeNotes({ bot, hands }: { bot: Bot; hands: Hands }) {
  const t = useT().browser;
  return (
    <div className="mt-2 flex flex-col items-center gap-2 text-center">
      <WindowAndWhy hands={hands} why={`${t.hands.takeWhy(bot.name)} ${t.window.why(bot.name)}`} />
    </div>
  );
}

/** Under the pill, while the owner has the browser. */
export function HeldNotes({
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
    <div className="mt-2 flex flex-col items-center gap-1.5 text-center text-xs">
      <p className="flex items-center gap-1.5 font-medium text-ink-soft">
        <Hint aria-hidden size={12} className="shrink-0" />
        {focused ? t.hands.typing : t.hands.clickToType}
      </p>
      <p className="line-clamp-2 max-w-md text-ink-soft" data-selectable>
        {task ? t.help.asks(bot.name, task) : t.hands.waits(bot.name)}
      </p>
      {tabs > 1 && <p className="text-muted">{t.hands.leaves(bot.name)}</p>}
      <div className="mt-1 flex flex-col items-center gap-2">
        <WindowAndWhy hands={hands} why={t.window.hint} />
      </div>
    </div>
  );
}
