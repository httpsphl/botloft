import { ArrowLeft, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotTemplate, BotTemplateFull } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { BotAvatar } from "../bots/BotAvatar";
import { Joined } from "./AgencyCard";
import type { RoleText } from "./roleText";

/**
 * "Learn more" about a role: the mascot with its name and the button that
 * adds it on top, then what it does, set to be read, and what the bot is told.
 */
export function RoleDetail({
  template,
  text,
  color,
  added,
  busy,
  onAdd,
  onBack,
  onCustomize,
}: {
  template: BotTemplate;
  text: RoleText;
  /** The mascot's color, the one it has in the list. */
  color: string;
  added: Bot | undefined;
  busy: boolean;
  onAdd(): void;
  onBack(): void;
  onCustomize(): void;
}) {
  const t = useT();
  const api = useApi();
  const words = t.catalog.detail;
  const [sheet, setSheet] = useState<BotTemplateFull | null>(null);

  useEffect(() => {
    let live = true;
    // Only for "Details": a failure just leaves it out.
    api.call("catalog.get", { id: template.id }).then(
      (full) => live && setSheet(full),
      () => {},
    );
    return () => {
      live = false;
    };
  }, [api, template.id]);

  // Side by side under what it does: when to call it, and who it works with.
  const asides: [string, string | null][] = [
    [words.when, text.when],
    [words.pairs, text.pairs],
  ];

  return (
    <section aria-label={text.name} className="flex flex-col gap-5">
      <div>
        <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
          {t.catalog.back}
        </Button>
      </div>
      <header className="flex flex-wrap items-center gap-4">
        <BotAvatar color={color} size={64} mood="awake" />
        <div className="min-w-0 flex-1">
          <p className="text-muted text-xs">{t.catalog.categories[template.category]}</p>
          <h3 className="font-semibold text-xl tracking-tight">{text.name}</h3>
          <p className="mt-0.5 text-ink-soft text-sm">{text.summary}</p>
        </div>
        <Button variant="primary" icon={Plus} disabled={busy} onClick={onAdd}>
          {t.catalog.add}
        </Button>
      </header>
      {added && <Joined bot={added} onCustomize={onCustomize} />}
      <div className="max-w-[65ch]">
        <h4 className="font-semibold text-sm">{words.what}</h4>
        <p className="mt-1 text-ink-soft text-sm leading-relaxed">{text.about ?? text.role}</p>
      </div>
      <div className="grid grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-3">
        {asides.map(
          ([title, body]) =>
            body && (
              <div key={title} className="rounded-xl bg-sunken/60 p-4">
                <h4 className="font-semibold text-sm">{title}</h4>
                <p className="mt-1 text-ink-soft text-sm leading-relaxed">{body}</p>
              </div>
            ),
        )}
      </div>
      {sheet && (
        <Details label={words.technical}>
          {`${sheet.model} · ${sheet.effort}\n\n${sheet.instructions}`}
        </Details>
      )}
    </section>
  );
}
