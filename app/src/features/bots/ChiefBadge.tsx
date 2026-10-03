// Marks the bot that leads its crew (spec 10.2): a crown, with "Chief" in
// full where there is room.

import { Crown } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { Badge } from "../../ui/Badge";

export const isChief = (bot: Bot, crew: Crew) => crew.leadBotId === bot.id;

export function ChiefBadge({ crew, compact = false }: { crew: Crew; compact?: boolean }) {
  const c = useT().bots.chief;
  if (compact) {
    return (
      <span title={c.hint(crew.name)} className="inline-flex shrink-0 text-accent">
        <Crown aria-hidden size={13} />
        <span className="sr-only">{c.badge}</span>
      </span>
    );
  }
  return (
    <Badge tone="accent" icon={Crown} title={c.hint(crew.name)}>
      {c.badge}
    </Badge>
  );
}
