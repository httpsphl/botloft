// The agent's shortcuts by its message field (spec 20.2): a play button
// that opens, upward, the routines that run when the owner clicks them.
// Nothing shows for an agent without one.

import { Play } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { useDismiss } from "../../ui/useDismiss";
import { shortcutsOf, useRunShortcut } from "./useShortcuts";

export function ShortcutMenu({ bot }: { bot: Bot }) {
  const words = useT().routines.shortcuts;
  const shortcuts = useApp(useShallow((state) => shortcutsOf(state, bot.id)));
  const run = useRunShortcut();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  useDismiss(
    open,
    root,
    useCallback(() => setOpen(false), []),
  );
  if (shortcuts.length === 0) {
    return null;
  }
  return (
    <div ref={root} className="relative">
      <button
        type="button"
        aria-label={words.menu(bot.name)}
        title={words.menu(bot.name)}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="grid h-8 w-8 shrink-0 place-items-center rounded-full text-muted hover:bg-sunken hover:text-ink"
      >
        <Play aria-hidden size={16} />
      </button>
      {open && (
        <div
          role="menu"
          aria-label={words.menu(bot.name)}
          className="absolute bottom-full left-0 z-20 mb-2 flex min-w-56 flex-col rounded-xl border border-line bg-panel p-1 shadow-lg"
        >
          {shortcuts.map((routine) => (
            <button
              key={routine.id}
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
                void run(routine, bot);
              }}
              className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm hover:bg-sunken"
            >
              <Play aria-hidden size={13} className="shrink-0 text-accent" />
              <span className="truncate">{routine.name}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
