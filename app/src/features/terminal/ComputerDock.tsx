// The dock of a bot's computer (spec 15.1): its browser, its terminal and
// its files, one click apart, at the foot of each of those panels. The
// three share one width (`computer`), so moving between them only changes
// what is inside.

import { Folder, Globe, type LucideIcon, SquareTerminal } from "lucide-react";
import { createContext, useContext } from "react";
import { useT } from "../../i18n";
import type { BotPanel } from "../../store/app";

type Place = Extract<BotPanel, "browser" | "terminal" | "files">;

/** The panel open and how to open another; absent outside a bot's view. */
export const Dock = createContext<{ open: BotPanel | null; pick(place: Place): void } | null>(null);

const PLACES: { place: Place; icon: LucideIcon }[] = [
  { place: "browser", icon: Globe },
  { place: "terminal", icon: SquareTerminal },
  { place: "files", icon: Folder },
];

/**
 * The dock as the foot of a panel: the same place in the browser, the
 * terminal and the files, so it stays still while the panel above changes.
 */
export function ComputerFooter() {
  const dock = useContext(Dock);
  if (!dock) {
    return null;
  }
  return (
    <div data-steady className="shrink-0 border-line border-t py-2">
      <ComputerDock />
    </div>
  );
}

export function ComputerDock({ className = "" }: { className?: string }) {
  const t = useT().terminal.dock;
  const dock = useContext(Dock);
  if (!dock) {
    return null;
  }
  return (
    <nav
      aria-label={t.label}
      className={`computer-dock mx-auto flex w-fit items-center gap-1 rounded-2xl p-1 ${className}`}
    >
      {PLACES.map(({ place, icon: Icon }) => {
        const open = dock.open === place;
        return (
          <button
            key={place}
            type="button"
            title={t[place]}
            aria-label={t[place]}
            aria-current={open ? "page" : undefined}
            onClick={() => dock.pick(place)}
            className={`grid size-9 place-items-center rounded-xl transition-[background-color,color,transform] hover:-translate-y-0.5 ${
              open ? "bg-panel text-ink shadow-sm" : "text-ink-soft hover:bg-panel/70"
            }`}
          >
            <Icon aria-hidden size={18} />
          </button>
        );
      })}
    </nav>
  );
}
