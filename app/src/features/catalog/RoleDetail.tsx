import { ArrowLeft, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BotTemplate, BotTemplateFull } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { Joined } from "./AgencyCard";
import type { RoleText } from "./roleText";

/** "Learn more" about a role: what it does, and what the bot is told. */
export function RoleDetail({
  template,
  text,
  added,
  busy,
  onAdd,
  onBack,
  onCustomize,
}: {
  template: BotTemplate;
  text: RoleText;
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

  const sections: [string, string | null][] = [
    [words.what, text.about ?? text.role],
    [words.when, text.when],
    [words.pairs, text.pairs],
  ];

  return (
    <section aria-label={text.name} className="flex flex-col gap-4">
      <div>
        <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
          {t.catalog.back}
        </Button>
      </div>
      <header>
        <h3 className="font-semibold text-lg tracking-tight">{text.name}</h3>
        <p className="mt-0.5 text-muted text-sm">{text.summary}</p>
      </header>
      {sections.map(
        ([title, body]) =>
          body && (
            <div key={title}>
              <h4 className="font-medium text-muted text-xs">{title}</h4>
              <p className="mt-0.5 text-sm">{body}</p>
            </div>
          ),
      )}
      {sheet && (
        <Details label={words.technical}>
          {`${sheet.model} · ${sheet.effort}\n\n${sheet.instructions}`}
        </Details>
      )}
      <div className="flex flex-wrap items-center gap-3">
        <Button variant="primary" icon={Plus} disabled={busy} onClick={onAdd}>
          {t.catalog.add}
        </Button>
        {added && <Joined bot={added} onCustomize={onCustomize} />}
      </div>
    </section>
  );
}
