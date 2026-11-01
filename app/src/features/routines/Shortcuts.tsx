// The shortcuts on the home (spec 15.1, 20.2): one button per routine that
// runs when the owner clicks it, from every crew. Nothing shows without one.

import { Play } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";
import { shortcutsOf, useRunShortcut } from "./useShortcuts";

export function Shortcuts() {
  const words = useT().routines.shortcuts;
  const shortcuts = useApp(useShallow((state) => shortcutsOf(state)));
  const bots = useApp((state) => state.bots);
  const run = useRunShortcut();
  if (shortcuts.length === 0) {
    return null;
  }
  return (
    <section aria-labelledby="home-shortcuts">
      <h2
        id="home-shortcuts"
        className="mb-3 flex items-center gap-1.5 font-semibold text-ink-soft text-sm"
      >
        <Play aria-hidden size={14} />
        {words.title}
      </h2>
      <ul className="flex flex-wrap gap-2">
        {shortcuts.map((routine) => {
          const bot = bots[routine.botId];
          return (
            <li key={routine.id}>
              <button
                type="button"
                aria-label={words.run(routine.name, bot?.name ?? "")}
                onClick={() => void run(routine, bot)}
                className="flex h-9 max-w-xs items-center gap-2 rounded-xl border border-line bg-panel px-3 text-sm transition-colors hover:border-line-strong hover:bg-sunken"
              >
                <Play aria-hidden size={13} className="shrink-0 text-accent" />
                <span className="truncate font-medium">{routine.name}</span>
                {bot && (
                  <span className="flex shrink-0 items-center gap-1 text-muted text-xs">
                    <BotAvatar color={bot.color} size={16} still botId={bot.id} />
                    {bot.name}
                  </span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
