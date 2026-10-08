// Settings (spec 15.1) as a page of its own: a search on top, the parts on
// the left in four groups (every day, account and connections, your data,
// Botloft) and the part open on the right.

import {
  Archive,
  Bell,
  DatabaseBackup,
  Info,
  type LucideIcon,
  MessageSquare,
  Palette,
  Plug,
  Search,
  SlidersHorizontal,
  Smartphone,
} from "lucide-react";
import { useId, useMemo, useState } from "react";
import { useT } from "../../i18n";
import { ConnectedToolsSettings } from "../connections/ConnectedToolsSettings";
import { AboutSettings } from "./AboutSettings";
import { AccountSettings } from "./AccountSettings";
import { AlertsSettings } from "./AlertsSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { ArchivedSettings } from "./ArchivedSettings";
import { BackupSettings } from "./BackupSettings";
import { ChatSettings } from "./ChatSettings";
import { GeneralSettings } from "./GeneralSettings";
import { type Page, plain, search, settingsIndex } from "./settingsIndex";

type GroupId = "groupDaily" | "groupConnect" | "groupData" | "groupBotloft";

const GROUPS: { id: GroupId; pages: { id: Page; icon: LucideIcon }[] }[] = [
  {
    id: "groupDaily",
    pages: [
      { id: "general", icon: SlidersHorizontal },
      { id: "appearance", icon: Palette },
      { id: "chat", icon: MessageSquare },
      { id: "alerts", icon: Bell },
    ],
  },
  {
    id: "groupConnect",
    pages: [
      { id: "account", icon: Smartphone },
      { id: "tools", icon: Plug },
    ],
  },
  {
    id: "groupData",
    pages: [
      { id: "backup", icon: DatabaseBackup },
      { id: "archived", icon: Archive },
    ],
  },
  { id: "groupBotloft", pages: [{ id: "about", icon: Info }] },
];

export function SettingsPage() {
  const t = useT();
  const s = t.account.settings;
  const [page, setPage] = useState<Page>("general");
  const [query, setQuery] = useState("");
  const id = useId();
  const names: Record<Page, string> = {
    general: s.general,
    appearance: s.appearance,
    chat: s.chat,
    alerts: s.alerts,
    account: s.account,
    tools: s.tools,
    backup: s.backup,
    archived: s.archived,
    about: s.about,
  };
  const entries = useMemo(() => settingsIndex(t), [t]);
  const found = search(entries, query, names);
  const searching = plain(query) !== "";

  const open = (to: Page) => {
    setPage(to);
    setQuery("");
  };

  return (
    <section aria-label={s.title} className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-6 px-5 py-6">
        <header className="flex flex-wrap items-center justify-between gap-3">
          <h1 className="font-semibold text-xl">{s.title}</h1>
          <label className="relative block w-full max-w-xs">
            <span className="sr-only">{s.search}</span>
            <Search
              aria-hidden
              size={14}
              className="-translate-y-1/2 pointer-events-none absolute top-1/2 left-2.5 text-muted"
            />
            <input
              type="search"
              value={query}
              placeholder={s.search}
              spellCheck={false}
              onChange={(event) => setQuery(event.target.value)}
              className="h-8 w-full rounded-lg border border-line-strong bg-canvas pr-2.5 pl-8 text-ink text-sm outline-none placeholder:text-muted focus:border-accent"
            />
          </label>
        </header>

        <div className="flex gap-6">
          <nav aria-label={s.pages} className="flex w-48 shrink-0 flex-col gap-0.5 self-start">
            <div role="tablist" aria-orientation="vertical" className="flex flex-col gap-0.5">
              {GROUPS.map((group) => (
                <div
                  key={group.id}
                  role="presentation"
                  className="flex flex-col gap-0.5 pt-4 first:pt-0"
                >
                  <p className="px-2.5 pb-1 font-semibold text-muted text-xs uppercase tracking-[0.12em]">
                    {s[group.id]}
                  </p>
                  {group.pages.map(({ id: each, icon: Icon }) => {
                    const selected = each === page && !searching;
                    return (
                      <button
                        key={each}
                        id={`${id}-${each}`}
                        type="button"
                        role="tab"
                        aria-selected={selected}
                        aria-controls={`${id}-panel`}
                        onClick={() => open(each)}
                        className={`flex min-h-8 items-center gap-2 rounded-lg px-2.5 py-1 text-left font-medium text-sm transition-colors ${
                          selected
                            ? "bg-sunken text-ink"
                            : "text-ink-soft hover:bg-sunken hover:text-ink"
                        }`}
                      >
                        <Icon aria-hidden size={15} className="shrink-0" />
                        {names[each]}
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>
          </nav>

          <div
            id={`${id}-panel`}
            role="tabpanel"
            aria-labelledby={`${id}-${page}`}
            className="flex min-w-0 flex-1 flex-col gap-6"
          >
            {searching ? (
              <Results found={found} query={query} names={names} open={open} />
            ) : (
              <>
                <h2 className="font-semibold text-lg">{names[page]}</h2>
                {page === "general" && <GeneralSettings />}
                {page === "appearance" && <AppearanceSettings />}
                {page === "chat" && <ChatSettings />}
                {page === "alerts" && <AlertsSettings />}
                {page === "account" && <AccountSettings />}
                {page === "tools" && <ConnectedToolsSettings />}
                {page === "backup" && <BackupSettings goToAccount={() => open("account")} />}
                {page === "archived" && <ArchivedSettings />}
                {page === "about" && <AboutSettings />}
              </>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}

function Results({
  found,
  query,
  names,
  open,
}: {
  found: ReturnType<typeof search>;
  query: string;
  names: Record<Page, string>;
  open(page: Page): void;
}) {
  const s = useT().account.settings;
  if (found.length === 0) {
    return <p className="text-muted text-sm">{s.searchNone(query.trim())}</p>;
  }
  return (
    <ul aria-label={s.search} className="flex flex-col divide-y divide-line">
      {found.map((entry) => (
        <li key={`${entry.page}/${entry.label}`}>
          <button
            type="button"
            onClick={() => open(entry.page)}
            className="flex w-full flex-col gap-0.5 rounded-lg px-2.5 py-2 text-left hover:bg-sunken"
          >
            <span className="font-medium text-sm">{entry.label}</span>
            <span className="text-muted text-xs">{s.searchIn(names[entry.page])}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}
