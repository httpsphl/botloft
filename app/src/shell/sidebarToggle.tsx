// Hiding the crews and bots on the left (spec 15.1), to keep only the chat
// and the bot's panel on a small screen. The choice is remembered.

import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { type ReactNode, useEffect } from "react";
import { useT } from "../i18n";
import { useApp } from "../store/context";
import { attentionCount } from "./attention";
import { prefs, usePref } from "./prefs";

export const toggleSidebar = () => prefs.sidebar.set(!prefs.sidebar.get());

/** The button in the title bar, with a dot while the list is hidden and something waits. */
export function SidebarToggle() {
  const t = useT().shell.sidebar;
  const open = usePref(prefs.sidebar);
  const waiting = useApp((state) => attentionCount(state) > 0);
  const Icon = open ? PanelLeftClose : PanelLeftOpen;
  return (
    <span className="relative grid">
      <button
        type="button"
        aria-label={open ? t.hide : t.show}
        title={`${open ? t.hide : t.show} (Ctrl+B)`}
        aria-pressed={!open}
        onClick={toggleSidebar}
        className="grid h-7 w-7 place-items-center rounded-md text-ink-soft hover:bg-sunken hover:text-ink"
      >
        <Icon aria-hidden size={15} />
      </button>
      {!open && waiting && (
        <span
          aria-hidden
          className="live-dot pointer-events-none absolute top-0.5 right-0.5"
          style={{ background: "var(--warn)" }}
        />
      )}
    </span>
  );
}

/** Holds the list, sliding it out to the left when hidden. */
export function SidebarSlot({ children }: { children: ReactNode }) {
  const open = usePref(prefs.sidebar);
  useSidebarKey();
  return (
    <div
      inert={!open}
      aria-hidden={!open}
      data-open={open}
      className="sidebar-slot flex shrink-0 overflow-hidden"
    >
      {children}
    </div>
  );
}

/** Ctrl+B shows or hides the list, as in code editors. */
function useSidebarKey(): void {
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!event.ctrlKey || event.altKey || event.metaKey || event.shiftKey) {
        return;
      }
      if (event.key.toLowerCase() === "b") {
        event.preventDefault();
        toggleSidebar();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}
