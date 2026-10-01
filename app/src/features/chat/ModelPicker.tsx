// Which Claude model the bot runs on (spec 7.4), picked below the chat as
// in Claude's apps. The plan's default shows the model Claude Code named.

import { ChevronDown } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { useT } from "../../i18n";
import { modelFamily, modelName } from "../../lib/models";
import type { Bot, BotModel } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { POPOVER } from "../../ui/surface";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";
import { PickerOption } from "./PickerOption";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

export function ModelPicker({ bot, onLater }: { bot: Bot; onLater(text: string): void }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const m = useT().chat.model;
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);

  const current = bot.model;
  const inUse = bot.modelInUse ? modelName(bot.modelInUse) : null;
  // The version once Claude Code has named it; until then, the choice.
  const label =
    current === "default"
      ? (inUse ?? m.short)
      : bot.modelInUse && modelFamily(bot.modelInUse) === current && inUse
        ? inUse
        : m.names[current];

  const choose = async (model: BotModel) => {
    setOpen(false);
    if (model === current) {
      return;
    }
    // The bot restarts on the new model once nothing is in progress.
    const working = bot.state === "busy" || bot.state === "needs_approval";
    await attempt(m.failed, async () => {
      putBot(await api.call("bots.setModel", { botId: bot.id, model }));
      if (working) {
        onLater(m.later(bot.name, m.names[model]));
      }
    });
  };

  const hint = (model: BotModel) =>
    model === "default" ? m.hints.default(bot.name, inUse) : m.hints[model](bot.name);

  return (
    <div ref={root} className="relative min-w-0">
      {open && (
        <div
          role="menu"
          aria-label={m.title}
          className={`absolute right-0 bottom-full mb-2 w-[22rem] max-w-[80vw] origin-bottom-right ${POPOVER}`}
        >
          <p className="px-2.5 pt-1.5 pb-1 font-medium text-muted text-xs">{m.title}</p>
          {MODELS.map((model) => (
            <PickerOption
              key={model}
              name={m.names[model]}
              hint={hint(model)}
              checked={current === model}
              onSelect={() => choose(model)}
            />
          ))}
          <p className="mx-1 mt-1 border-line border-t px-1.5 pt-2 pb-1.5 text-muted text-xs">
            {m.cost}
          </p>
        </div>
      )}
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={m.button(label)}
        title={hint(current)}
        onClick={() => setOpen(!open)}
        className={`flex h-8 max-w-full items-center gap-1 rounded-full px-2.5 text-sm text-ink-soft transition-colors hover:bg-sunken hover:text-ink ${
          open ? "bg-sunken text-ink" : ""
        }`}
      >
        <span className="truncate">{label}</span>
        <ChevronDown
          aria-hidden
          size={14}
          className={`hidden shrink-0 opacity-60 transition-transform duration-200 @xs:block ${open ? "rotate-180" : ""}`}
        />
      </button>
    </div>
  );
}
