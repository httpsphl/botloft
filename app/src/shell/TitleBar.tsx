// The window's own title bar (spec 15.3: `decorations: false`), with the
// Windows controls on the right. Empty space drags the window.

import { ALargeSmall, Copy, Languages, Minus, Monitor, Moon, Square, Sun, X } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { LOCALES, setLocaleChoice, systemLocale, useLocale, useT } from "../i18n";
import { useHost } from "../store/context";
import { Menu } from "../ui/Menu";
import { setTheme, type ThemeChoice, useTheme } from "./theme";
import { DEFAULT_ZOOM, setZoom, useZoom, ZOOM_LEVELS } from "./zoom";

const NEXT_THEME: Record<ThemeChoice, ThemeChoice> = {
  system: "light",
  light: "dark",
  dark: "system",
};
const THEME_ICON = { system: Monitor, light: Sun, dark: Moon };

export function TitleBar({ children, status }: { children?: ReactNode; status?: ReactNode }) {
  const t = useT();
  const { choice } = useTheme();
  const ThemeIcon = THEME_ICON[choice];
  const themeName = t.shell.theme[choice];
  return (
    <header
      data-tauri-drag-region
      className="flex h-9 shrink-0 items-center border-line border-b bg-panel"
    >
      <div data-tauri-drag-region className="flex items-center gap-2 pr-4 pl-3">
        <BotAvatar color="#ffffff" size={18} framed />
        <span data-tauri-drag-region className="font-semibold text-sm tracking-tight">
          Botloft
        </span>
      </div>
      <div data-tauri-drag-region className="flex min-w-0 flex-1 items-center gap-1.5 text-sm">
        {children}
      </div>
      <div className="flex items-center gap-1 pr-2">
        {status}
        <ZoomMenu />
        <LanguageMenu />
        <button
          type="button"
          aria-label={t.shell.theme.label(themeName)}
          title={t.shell.theme.hint(themeName)}
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

/** How big the app is drawn (spec 15.3); Ctrl+= and Ctrl+- do the same. */
function ZoomMenu() {
  const t = useT();
  const zoom = useZoom();
  return (
    <Menu
      label={t.shell.zoom.label}
      icon={ALargeSmall}
      items={ZOOM_LEVELS.map((level) => ({
        label: t.shell.zoom.level(Math.round(level * 100), level === DEFAULT_ZOOM),
        checked: zoom === level,
        onSelect: () => setZoom(level),
      }))}
    />
  );
}

/** Picks the app's language; "system" follows Windows (spec 15.6). */
function LanguageMenu() {
  const t = useT();
  const { choice } = useLocale();
  const system = LOCALES.find((locale) => locale.id === systemLocale())?.name ?? "English";
  return (
    <Menu
      label={t.shell.language.label}
      icon={Languages}
      items={[
        {
          label: t.shell.language.system(system),
          checked: choice === "system",
          onSelect: () => setLocaleChoice("system"),
        },
        ...LOCALES.map((locale) => ({
          label: locale.name,
          checked: choice === locale.id,
          onSelect: () => setLocaleChoice(locale.id),
        })),
      ]}
    />
  );
}

function WindowControls() {
  const t = useT();
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
        aria-label={t.shell.window.minimize}
        onClick={() => window.minimize()}
        className={`${control} hover:bg-sunken`}
      >
        <Minus aria-hidden size={15} strokeWidth={1.5} />
      </button>
      <button
        type="button"
        aria-label={maximized ? t.shell.window.restore : t.shell.window.maximize}
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
        aria-label={t.shell.window.close}
        onClick={() => window.close()}
        className={`${control} hover:bg-[#c42b1c] hover:text-white`}
      >
        <X aria-hidden size={16} strokeWidth={1.5} />
      </button>
    </div>
  );
}
