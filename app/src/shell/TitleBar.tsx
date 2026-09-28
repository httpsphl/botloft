// The window's own title bar (spec 15.3: `decorations: false`), with the
// Windows controls on the right. Empty space drags the window.

import { Copy, Minus, Monitor, Moon, Square, Sun, X } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useHost } from "../store/context";
import { setTheme, type ThemeChoice, useTheme } from "./theme";

const NEXT_THEME: Record<ThemeChoice, ThemeChoice> = {
  system: "light",
  light: "dark",
  dark: "system",
};
const THEME_ICON = { system: Monitor, light: Sun, dark: Moon };

export function TitleBar({ children, status }: { children?: ReactNode; status?: ReactNode }) {
  const { choice } = useTheme();
  const ThemeIcon = THEME_ICON[choice];
  return (
    <header
      data-tauri-drag-region
      className="flex h-9 shrink-0 items-center border-line border-b bg-panel"
    >
      <div data-tauri-drag-region className="flex items-center gap-2 pr-4 pl-3">
        <BotAvatar color="#ffffff" size={18} />
        <span data-tauri-drag-region className="font-semibold text-sm tracking-tight">
          Botloft
        </span>
      </div>
      <div data-tauri-drag-region className="flex min-w-0 flex-1 items-center gap-1.5 text-sm">
        {children}
      </div>
      <div className="flex items-center gap-1 pr-2">
        {status}
        <button
          type="button"
          aria-label={`Theme: ${choice}`}
          title={`Theme: ${choice} (click to change)`}
          onClick={() => setTheme(NEXT_THEME[choice])}
          className="grid h-7 w-7 place-items-center text-muted hover:bg-sunken hover:text-ink"
        >
          <ThemeIcon aria-hidden size={14} />
        </button>
      </div>
      <WindowControls />
    </header>
  );
}

function WindowControls() {
  const { window } = useHost();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let alive = true;
    let stop: (() => void) | undefined;
    const refresh = () => {
      window.isMaximized().then(
        (value) => alive && setMaximized(value),
        () => {},
      );
    };
    refresh();
    window.onResized(refresh).then(
      (unsubscribe) => {
        stop = unsubscribe;
        if (!alive) {
          unsubscribe();
        }
      },
      () => {},
    );
    return () => {
      alive = false;
      stop?.();
    };
  }, [window]);

  const control = "grid h-9 w-11 place-items-center text-ink-soft";
  return (
    <div className="flex self-stretch">
      <button
        type="button"
        aria-label="Minimize"
        onClick={() => window.minimize()}
        className={`${control} hover:bg-sunken`}
      >
        <Minus aria-hidden size={15} strokeWidth={1.5} />
      </button>
      <button
        type="button"
        aria-label={maximized ? "Restore" : "Maximize"}
        onClick={() => window.toggleMaximize()}
        className={`${control} hover:bg-sunken`}
      >
        {maximized ? (
          <Copy aria-hidden size={12} strokeWidth={1.5} className="-scale-x-100" />
        ) : (
          <Square aria-hidden size={12} strokeWidth={1.5} />
        )}
      </button>
      <button
        type="button"
        aria-label="Close"
        onClick={() => window.close()}
        className={`${control} hover:bg-[#c42b1c] hover:text-white`}
      >
        <X aria-hidden size={16} strokeWidth={1.5} />
      </button>
    </div>
  );
}
