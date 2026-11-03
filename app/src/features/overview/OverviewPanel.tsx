// The overview beside an agent's chat (spec 33): what fills its
// conversation now. What runs in the background, what changed in its
// folders and its connected tools come in later parts of the panel.

import { X } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { SidePanel } from "../../ui/SidePanel";
import { ContextSection } from "./ContextSection";

export function OverviewPanel({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const t = useT().overview;
  return (
    <SidePanel label={t.panel(bot.name)} name="overview" defaultWidth={360}>
      <header className="flex h-11 shrink-0 items-center justify-between pr-1.5 pl-4">
        <h2 className="font-semibold text-base">{t.heading}</h2>
        <Button variant="ghost" size="sm" icon={X} label={t.close} onClick={onClose} />
      </header>
      <div className="flex min-h-0 flex-col gap-6 overflow-y-auto px-4 pt-1 pb-4">
        <ContextSection bot={bot} />
      </div>
    </SidePanel>
  );
}
