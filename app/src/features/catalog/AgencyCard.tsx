import { Check, Plus } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, BotTemplate } from "../../lib/protocol.gen";
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

/**
 * A role as a line of the list: its mascot, name, kind and what it does,
 * which opens the whole of it, and a light button that adds it.
 */
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
    <li className="flex flex-col gap-1.5 rounded-xl p-2 transition-colors hover:bg-sunken/60">
      <div className="flex items-center gap-3">
        <button
          type="button"
          aria-label={`${t.catalog.learnMore}: ${text.name}`}
          title={t.catalog.learnMore}
          onClick={onLearnMore}
          className="flex min-w-0 flex-1 items-center gap-3 rounded-lg text-left"
        >
          <BotAvatar color={color} size={40} />
          <span className="min-w-0 flex-1">
            <span className="flex items-baseline gap-2">
              <span className="truncate font-semibold">{text.name}</span>
              <span className="shrink-0 text-muted text-xs">
                {t.catalog.categories[template.category]}
              </span>
            </span>
            <span className="line-clamp-2 text-muted text-sm">{text.summary}</span>
          </span>
        </button>
        <button
          type="button"
          disabled={busy}
          aria-label={`${t.catalog.add}: ${text.name}`}
          onClick={onAdd}
          className="flex h-8 shrink-0 items-center gap-1 rounded-full bg-sunken px-3 font-medium text-ink text-sm transition-[background-color,transform] duration-150 hover:bg-line active:scale-[0.97] disabled:opacity-45"
        >
          <Plus aria-hidden size={14} />
          {t.catalog.addShort}
        </button>
      </div>
      {added && (
        <div className="pl-[3.25rem]">
          <Joined bot={added} onCustomize={onCustomize} />
        </div>
      )}
    </li>
  );
}
