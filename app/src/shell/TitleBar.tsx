// The window's own title bar (spec 15.3: `decorations: false`), with the
// Windows controls on the right. Empty space drags the window. Theme, size
// and language live in the account area of the sidebar (spec 15.1).

import { Copy, Languages, Minus, Square, X } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { BOTLOFT_COLOR, BotAvatar } from "../features/bots/BotAvatar";
import { LOCALES, setLocaleChoice, systemLocale, useLocale, useT } from "../i18n";
import { useHost } from "../store/context";
import { Menu } from "../ui/Menu";

export function TitleBar({
  children,
  status,
  language = false,
}: {
  children?: ReactNode;
  status?: ReactNode;
  /** Shows the language menu, for the setup screens that have no sidebar. */
  language?: boolean;
}) {
  return (
    <header
      data-tauri-drag-region
      className="flex h-9 shrink-0 items-center border-line border-b bg-panel"
    >
      <div data-tauri-drag-region className="flex items-center gap-2 pr-4 pl-3">
        <BotAvatar color={BOTLOFT_COLOR} size={18} framed />
        <span data-tauri-drag-region className="font-semibold text-sm tracking-tight">
          Botloft
        </span>
      </div>
      <div data-tauri-drag-region className="flex min-w-0 flex-1 items-center gap-1.5 text-sm">
        {children}
      </div>
      <div className="flex items-center gap-1 pr-2">
        {status}
        {language && <LanguageMenu />}
      </div>
      <WindowControls />
    </header>
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
