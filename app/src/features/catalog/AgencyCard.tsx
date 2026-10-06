import { Check, Plus } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, BotTemplate } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { BotAvatar } from "../bots/BotAvatar";
import type { RoleText } from "./roleText";

/** What a role just did: "<name> joined the crew", and a way to change it. */
export function Joined({ bot, onCustomize }: { bot: Bot; onCustomize(): void }) {
  const t = useT();
  return (
    <p className="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
      <span className="flex items-center gap-1 font-medium text-ok">
        <Check aria-hidden size={14} />
        {t.catalog.joined(bot.name)}
      </span>
      <button
        type="button"
        onClick={onCustomize}
        className="font-medium text-ink-soft underline-offset-2 hover:text-ink hover:underline"
      >
        {t.catalog.customize}
      </button>
    </p>
  );
}

/** A role as a card: what it is, and the two things to do with it. */
export function AgencyCard({
  template,
  text,
  color,
  added,
  busy,
  onAdd,
  onLearnMore,
  onCustomize,
}: {
  template: BotTemplate;
  text: RoleText;
  color: string;
  added: Bot | undefined;
  busy: boolean;
  onAdd(): void;
  onLearnMore(): void;
  onCustomize(): void;
}) {
  const t = useT();
  return (
    <li className="flex flex-col gap-2 rounded-xl border border-line bg-panel p-3.5">
      <div className="flex items-center gap-2.5">
        <BotAvatar color={color} size={32} />
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-semibold">{text.name}</h3>
          <p className="truncate text-muted text-xs">{t.catalog.categories[template.category]}</p>
        </div>
      </div>
      <p className="line-clamp-2 min-h-10 text-ink-soft text-sm">{text.summary}</p>
      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          size="sm"
          icon={Plus}
          disabled={busy}
          aria-label={`${t.catalog.add}: ${text.name}`}
          onClick={onAdd}
        >
          {t.catalog.add}
        </Button>
        <Button size="sm" aria-label={`${t.catalog.learnMore}: ${text.name}`} onClick={onLearnMore}>
          {t.catalog.learnMore}
        </Button>
      </div>
      {added && <Joined bot={added} onCustomize={onCustomize} />}
    </li>
  );
}
