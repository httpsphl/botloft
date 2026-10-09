// The model of a bot that does not run on Claude Code (spec 30), picked below
// the chat like Claude's. The agent lists its models; its effort is part of
// each name ("... (High)"), so there is no second picker.

import { ChevronDown } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { AgentModel, Bot } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { POPOVER } from "../../ui/surface";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";
import { PickerOption } from "./PickerOption";

export function AgentModelPicker({ bot, onLater }: { bot: Bot; onLater(text: string): void }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const m = useT().chat.agentModel;
  const [open, setOpen] = useState(false);
  const [models, setModels] = useState<AgentModel[] | null>(null);
  const [failed, setFailed] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);

  // Asked when the list is opened first: it takes a few seconds.
  useEffect(() => {
    if (!open || models !== null) {
      return;
    }
    let live = true;
    api
      .call("agents.models", { agent: bot.agent })
      .then((list) => live && setModels(list))
      .catch(() => live && setFailed(true));
    return () => {
      live = false;
    };
  }, [open, models, api, bot.agent]);

  const current = bot.agentModel;
  const nameOf = (id: string) => models?.find((model) => model.id === id)?.name ?? id;
  const label =
    current === null ? (bot.modelInUse ? nameOf(bot.modelInUse) : m.short) : nameOf(current);

  const choose = async (model: string | null) => {
    setOpen(false);
    if (model === current) {
      return;
    }
    const working = bot.state === "busy" || bot.state === "needs_approval";
    await attempt(m.failed, async () => {
      putBot(await api.call("bots.setAgentModel", { botId: bot.id, model }));
      if (working) {
        onLater(m.later(bot.name, model === null ? m.short : nameOf(model)));
      }
    });
  };

  return (
    <div ref={root} className="relative min-w-0">
      {open && (
        <div
          role="menu"
          aria-label={m.title}
          className={`absolute right-0 bottom-full mb-2 max-h-80 w-[22rem] max-w-[80vw] origin-bottom-right overflow-y-auto ${POPOVER}`}
        >
          <p className="px-2.5 pt-1.5 pb-1 font-medium text-muted text-xs">{m.title}</p>
          <PickerOption
            name={m.short}
            hint={m.defaultHint(bot.name)}
            checked={current === null}
            onSelect={() => choose(null)}
          />
          {models === null && !failed && (
            <p className="px-2.5 py-2 text-muted text-xs">{m.loading}</p>
          )}
          {failed && <p className="px-2.5 py-2 text-danger text-xs">{m.listFailed}</p>}
          {models?.map((model) => (
            <PickerOption
              key={model.id}
              name={model.name}
              hint={model.id}
              checked={current === model.id}
              onSelect={() => choose(model.id)}
            />
          ))}
        </div>
      )}
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={m.button(label)}
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
