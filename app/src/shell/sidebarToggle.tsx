// Hiding the crews and bots on the left (spec 15.1), to keep only the chat
// and the bot's panel on a small screen. The choice is remembered.

import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { type ReactNode, useEffect } from "react";
import { useT } from "../i18n";
import { useApp } from "../store/context";
import { anyUnread } from "../store/seen";
import { attentionCount } from "./attention";
import { prefs, usePref } from "./prefs";

export const toggleSidebar = () => prefs.sidebar.set(!prefs.sidebar.get());

/**
 * The button in the title bar. While the list is hidden, a dot says
 * something waits for the owner, or, quieter, that a bot replied.
 */
export function SidebarToggle() {
  const t = useT().shell.sidebar;
  const open = usePref(prefs.sidebar);
  const waiting = useApp((state) => attentionCount(state) > 0);
  const unread = useApp(anyUnread);
  const Icon = open ? PanelLeftClose : PanelLeftOpen;
  return (
    <span className="relative grid">
      <button
        type="button"
        aria-label={open ? t.hide : t.show}
        title={`${open ? t.hide : t.show} (Ctrl+B)`}
        aria-pressed={!open}
        onClick={toggleSidebar}
        className="grid h-7 w-7 place-items-center rounded-lg text-ink-soft hover:bg-sunken hover:text-ink"
      >
        <Icon aria-hidden size={15} />
      </button>
      {!open && (waiting || unread) && (
        <span
          aria-hidden
          className={`pointer-events-none absolute top-0.5 right-0.5 ${waiting ? "live-dot" : "size-[7px] rounded-full"}`}
          style={{ background: waiting ? "var(--warn)" : "var(--accent)" }}
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
    <div inert={!open} aria-hidden={!open} data-open={open} className="sidebar-slot shrink-0">
      <div className="sidebar-slide">{children}</div>
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
