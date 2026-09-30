// Settings (spec 15.1), in parts listed on the left: general (what
// Botloft does in the background, the language), the chat, notifications,
// the look, what was archived, and the versions for when something needs
// reporting.

import {
  Archive,
  Bell,
  Info,
  type LucideIcon,
  MessageSquare,
  Palette,
  SlidersHorizontal,
} from "lucide-react";
import { useId, useState } from "react";
import { useT } from "../../i18n";
import { Dialog } from "../../ui/Dialog";
import { AboutSettings } from "./AboutSettings";
import { AlertsSettings } from "./AlertsSettings";
import { AppearanceSettings } from "./AppearanceSettings";
import { ArchivedSettings } from "./ArchivedSettings";
import { ChatSettings } from "./ChatSettings";
import { GeneralSettings } from "./GeneralSettings";

type Page = "general" | "chat" | "alerts" | "appearance" | "archived" | "about";

const PAGES: { id: Page; icon: LucideIcon }[] = [
  { id: "general", icon: SlidersHorizontal },
  { id: "chat", icon: MessageSquare },
  { id: "alerts", icon: Bell },
  { id: "appearance", icon: Palette },
  { id: "archived", icon: Archive },
  { id: "about", icon: Info },
];

export function SettingsDialog({ onClose }: { onClose(): void }) {
  const s = useT().account.settings;
  const [page, setPage] = useState<Page>("general");
  const id = useId();

  return (
    <Dialog title={s.title} onClose={onClose} width="lg">
      <div className="flex min-h-[27rem] gap-5">
        <div
          role="tablist"
          aria-label={s.pages}
          aria-orientation="vertical"
          className="flex w-40 shrink-0 flex-col gap-0.5"
        >
          {PAGES.map(({ id: each, icon: Icon }) => {
            const selected = each === page;
            return (
              <button
                key={each}
                id={`${id}-${each}`}
                type="button"
                role="tab"
                aria-selected={selected}
                aria-controls={`${id}-panel`}
                onClick={() => setPage(each)}
                className={`flex h-8 items-center gap-2 rounded-lg px-2.5 text-left font-medium text-sm transition-colors ${
                  selected ? "bg-sunken text-ink" : "text-ink-soft hover:bg-sunken hover:text-ink"
                }`}
              >
                <Icon aria-hidden size={15} className="shrink-0" />
                {s[each]}
              </button>
            );
          })}
        </div>
        <div
          id={`${id}-panel`}
          role="tabpanel"
          aria-labelledby={`${id}-${page}`}
          className="flex min-w-0 flex-1 flex-col gap-6"
        >
          {page === "general" && <GeneralSettings />}
          {page === "chat" && <ChatSettings />}
          {page === "alerts" && <AlertsSettings />}
          {page === "appearance" && <AppearanceSettings />}
          {page === "archived" && <ArchivedSettings />}
          {page === "about" && <AboutSettings />}
        </div>
      </div>
    </Dialog>
  );
}
