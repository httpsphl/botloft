// The owner at the bottom of the sidebar, like the account corner of a
// chat app: their name and Claude plan, and a menu with usage, settings,
// language, what's new and help (spec 15.1).

import {
  ChartColumn,
  Check,
  ChevronRight,
  ChevronsUpDown,
  CircleHelp,
  Languages,
  Settings,
  Sparkles,
} from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { LOCALES, setLocaleChoice, systemLocale, useLocale, useT } from "../../i18n";
import type { ClaudeAccount } from "../../lib/protocol.gen";
import { useApp, useHost } from "../../store/context";
import { attempt } from "../../ui/toast";
import { SettingsDialog } from "./SettingsDialog";
import { UsageDialog } from "./UsageDialog";

const RELEASES = "https://github.com/httpsphl/botloft/releases";
const HELP = "https://github.com/httpsphl/botloft#readme";

/** "max" reads "Max"; an unknown plan keeps its words. */
function planName(plan: string): string {
  const words = plan.replaceAll("_", " ").trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

export function AccountArea() {
  const a = useT().account;
  const account = useApp((state) => state.system?.account ?? null);
  const signedOut = useApp((state) => state.system?.claudeSignedIn === false);
  const [open, setOpen] = useState(false);
  const [dialog, setDialog] = useState<"usage" | "settings" | null>(null);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && setOpen(false);
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const name = account?.name.trim() || "Botloft";
  const detail = describe(account?.claude ?? null, signedOut, a);
  const show = (which: "usage" | "settings") => {
    setOpen(false);
    setDialog(which);
  };

  return (
    <div ref={root} className="relative shrink-0 border-line border-t p-2">
      {open && (
        <AccountMenu
          name={name}
          email={account?.claude?.email ?? null}
          onShow={show}
          onClose={() => setOpen(false)}
        />
      )}
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={a.open(name)}
        onClick={() => setOpen(!open)}
        className={`flex w-full items-center gap-2.5 px-2 py-1.5 text-left hover:bg-sunken ${open ? "bg-sunken" : ""}`}
      >
        <span
          aria-hidden
          className="grid h-8 w-8 shrink-0 place-items-center rounded-full bg-ink font-semibold text-canvas text-sm"
        >
          {Array.from(name)[0]?.toUpperCase()}
        </span>
        <span className="min-w-0 flex-1">
          <span className="block truncate font-medium text-sm">{name}</span>
          {detail && <span className="block truncate text-muted text-xs">{detail}</span>}
        </span>
        <ChevronsUpDown aria-hidden size={14} className="shrink-0 text-muted" />
      </button>
      {dialog === "usage" && <UsageDialog onClose={() => setDialog(null)} />}
      {dialog === "settings" && <SettingsDialog onClose={() => setDialog(null)} />}
    </div>
  );
}

function describe(
  claude: ClaudeAccount | null,
  signedOut: boolean,
  a: ReturnType<typeof useT>["account"],
): string {
  if (claude?.plan) {
    return a.plan(planName(claude.plan));
  }
  return claude?.organization ?? claude?.email ?? (signedOut ? a.notSignedIn : "");
}

function AccountMenu({
  name,
  email,
  onShow,
  onClose,
}: {
  name: string;
  email: string | null;
  onShow(which: "usage" | "settings"): void;
  onClose(): void;
}) {
  const t = useT();
  const m = t.account.menu;
  const host = useHost();
  const { choice } = useLocale();
  const [languages, setLanguages] = useState(false);
  const systemName = LOCALES.find((entry) => entry.id === systemLocale())?.name ?? "English";
  const open = (url: string) => {
    onClose();
    return attempt(m.openFailed, () => host.openUrl(url));
  };

  return (
    <div
      role="menu"
      aria-label={name}
      className="absolute right-2 bottom-full left-2 z-30 mb-1 border border-line-strong bg-panel py-1"
    >
      {email && (
        <p className="truncate px-3 pt-1 pb-2 text-muted text-xs" data-selectable>
          {email}
        </p>
      )}
      <Item icon={ChartColumn} onClick={() => onShow("usage")}>
        {m.usage}
      </Item>
      <Item icon={Settings} onClick={() => onShow("settings")}>
        {m.settings}
      </Item>
      <div className="relative">
        <Item
          icon={Languages}
          onClick={() => setLanguages(!languages)}
          expanded={languages}
          trailing={<ChevronRight aria-hidden size={14} className="text-muted" />}
        >
          {m.language}
        </Item>
        {languages && (
          <div
            role="menu"
            aria-label={m.language}
            className="absolute bottom-0 left-full z-30 ml-1 min-w-56 border border-line-strong bg-panel py-1"
          >
            {[{ id: "system" as const, name: t.shell.language.system(systemName) }, ...LOCALES].map(
              (entry) => (
                <button
                  key={entry.id}
                  type="button"
                  role="menuitemradio"
                  aria-checked={choice === entry.id}
                  onClick={() => setLocaleChoice(entry.id)}
                  className="flex h-8 w-full items-center gap-2 whitespace-nowrap px-3 text-left text-sm hover:bg-sunken"
                >
                  <Check aria-hidden size={14} className={choice === entry.id ? "" : "invisible"} />
                  {entry.name}
                </button>
              ),
            )}
          </div>
        )}
      </div>
      <div className="my-1 border-line border-t" />
      <Item icon={Sparkles} onClick={() => open(RELEASES)}>
        {m.whatsNew}
      </Item>
      <Item icon={CircleHelp} onClick={() => open(HELP)}>
        {m.help}
      </Item>
    </div>
  );
}

function Item({
  icon: Icon,
  onClick,
  expanded,
  trailing,
  children,
}: {
  icon: typeof Settings;
  onClick(): void;
  expanded?: boolean;
  trailing?: ReactNode;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      role="menuitem"
      aria-haspopup={expanded === undefined ? undefined : "menu"}
      aria-expanded={expanded}
      onClick={onClick}
      className="flex h-8 w-full items-center gap-2 px-3 text-left text-ink text-sm hover:bg-sunken"
    >
      <Icon aria-hidden size={14} className="text-ink-soft" />
      <span className="flex-1">{children}</span>
      {trailing}
    </button>
  );
}
