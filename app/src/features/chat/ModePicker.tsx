// How much the bot may do without asking (spec 7.4), picked below the chat
// as in Claude Code. Bypassing permissions asks first: the bot is then not
// held to its folder.

import {
  ChevronDown,
  FilePen,
  Hand,
  ListTodo,
  type LucideIcon,
  ShieldOff,
  WandSparkles,
} from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, PermissionMode } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { POPOVER } from "../../ui/surface";
import { attempt } from "../../ui/toast";
import { useDismiss } from "../../ui/useDismiss";
import { PickerOption } from "./PickerOption";

const MODES: { mode: PermissionMode; icon: LucideIcon }[] = [
  { mode: "auto", icon: WandSparkles },
  { mode: "default", icon: Hand },
  { mode: "accept_edits", icon: FilePen },
  { mode: "plan", icon: ListTodo },
];

export function ModePicker({ bot, onLater }: { bot: Bot; onLater(text: string): void }) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const chief = useApp((state) => state.crews[bot.crewId]?.leadBotId === bot.id);
  const m = useT().chat.mode;
  const [open, setOpen] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);

  const current = bot.permissionMode;
  const bypass = current === "bypass_permissions";
  const Icon = bypass ? ShieldOff : (MODES.find((entry) => entry.mode === current)?.icon ?? Hand);

  const choose = async (mode: PermissionMode) => {
    setOpen(false);
    if (mode === current) {
      return;
    }
    // The bot restarts into the new mode once nothing is in progress.
    const working = bot.state === "busy" || bot.state === "needs_approval";
    await attempt(m.failed, async () => {
      putBot(await api.call("bots.setPermissionMode", { botId: bot.id, mode }));
      if (working) {
        onLater(m.later(bot.name, m.names[mode]));
      }
    });
  };

  return (
    <div ref={root} className="relative shrink-0">
      {open && (
        <div
          role="menu"
          aria-label={m.title}
          className={`absolute bottom-full left-0 mb-2 w-[22rem] max-w-[80vw] origin-bottom-left ${POPOVER}`}
        >
          <p className="px-2.5 pt-1.5 pb-1 font-medium text-muted text-xs">{m.title}</p>
          {MODES.map(({ mode, icon }) => (
            <PickerOption
              key={mode}
              icon={icon}
              name={m.names[mode]}
              hint={m.hints[mode](bot.name)}
              checked={current === mode}
              onSelect={() => choose(mode)}
            />
          ))}
          <div className="mx-1 my-1 border-line border-t" />
          <PickerOption
            icon={ShieldOff}
            danger
            name={m.names.bypass_permissions}
            hint={m.hints.bypass_permissions(bot.name)}
            checked={bypass}
            onSelect={() => {
              setOpen(false);
              if (!bypass) {
                setConfirming(true);
              }
            }}
            trailing={
              <span className="shrink-0 rounded-lg border border-danger/50 px-2 py-0.5 font-medium text-danger text-xs">
                {m.turnOn}
              </span>
            }
          />
        </div>
      )}
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={m.button(m.names[current])}
        title={m.hints[current](bot.name)}
        onClick={() => setOpen(!open)}
        className={`flex h-8 items-center gap-1.5 rounded-full px-2.5 text-sm transition-colors ${
          bypass
            ? "bg-danger/10 text-danger hover:bg-danger/15"
            : `text-ink-soft hover:bg-sunken hover:text-ink ${open ? "bg-sunken text-ink" : ""}`
        }`}
      >
        <Icon aria-hidden size={15} />
        <span className="hidden whitespace-nowrap @sm:inline">{m.names[current]}</span>
        <ChevronDown
          aria-hidden
          size={14}
          className={`hidden opacity-60 transition-transform duration-200 @sm:block ${open ? "rotate-180" : ""}`}
        />
      </button>
      {confirming && (
        <Confirm
          title={m.bypassTitle(bot.name)}
          confirmLabel={m.turnOn}
          onConfirm={() => choose("bypass_permissions")}
          onClose={() => setConfirming(false)}
        >
          <p>{m.bypassBody(bot.name)}</p>
          <p className="mt-2">{m.bypassRisk}</p>
          {chief && <p className="mt-2">{m.bypassChief(bot.name)}</p>}
          <p className="mt-2 font-medium text-ink">{m.bypassAdvice}</p>
        </Confirm>
      )}
    </div>
  );
}
