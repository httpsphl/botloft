import { ChevronLeft } from "lucide-react";
import { useT } from "../../i18n";
import type { BotTemplate } from "../../lib/protocol.gen";
import { roleText } from "../catalog/roleText";
import { CREW_TEMPLATES, type CrewTemplate } from "./crewTemplates";

/** The names of a template's bots, in the owner's language. */
export function roleNames(
  t: ReturnType<typeof useT>,
  template: CrewTemplate,
  roles: readonly BotTemplate[],
): string[] {
  return template.roles.flatMap((id) => {
    const role = roles.find((candidate) => candidate.id === id);
    return role ? [roleText(t, role).name] : [];
  });
}

/** The first step of a new crew: a ready-made team, or an empty crew. */
export function TemplatePicker({
  onPick,
  onScratch,
}: {
  onPick(template: CrewTemplate): void;
  onScratch(): void;
}) {
  const t = useT();
  const text = t.crewTemplates;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-ink-soft text-sm">{text.intro}</p>
      <ul className="m-0 grid list-none grid-cols-1 gap-2 p-0 sm:grid-cols-2">
        {CREW_TEMPLATES.map((template) => {
          const item = text.items[template.id];
          return (
            <li key={template.id} className="flex">
              <button
                type="button"
                onClick={() => onPick(template)}
                className="flex w-full flex-col gap-1 rounded-xl border border-line bg-panel p-3 text-left transition-colors hover:border-line-strong hover:bg-sunken"
              >
                <span className="font-semibold">{item.name}</span>
                <span className="line-clamp-2 text-ink-soft text-sm">{item.summary}</span>
                <span className="text-muted text-xs">{text.bots(template.roles.length)}</span>
              </button>
            </li>
          );
        })}
      </ul>
      <button
        type="button"
        onClick={onScratch}
        className="flex flex-col gap-0.5 rounded-xl border border-line border-dashed p-3 text-left transition-colors hover:border-line-strong hover:bg-sunken"
      >
        <span className="font-semibold">{text.scratch}</span>
        <span className="text-ink-soft text-sm">{text.scratchHint}</span>
      </button>
    </div>
  );
}

/** What a picked template adds, with a way back to the list. */
export function PickedTemplate({
  template,
  roles,
  onChange,
}: {
  template: CrewTemplate;
  roles: readonly BotTemplate[];
  onChange(): void;
}) {
  const t = useT();
  const text = t.crewTemplates;
  return (
    <div className="flex flex-col gap-1.5 rounded-xl border border-line bg-sunken p-3 text-sm">
      <div className="flex items-center justify-between gap-2">
        <span className="font-semibold">{text.items[template.id].name}</span>
        <button
          type="button"
          onClick={onChange}
          className="flex items-center gap-0.5 font-medium text-ink-soft hover:text-ink"
        >
          <ChevronLeft aria-hidden size={14} />
          {text.change}
        </button>
      </div>
      <p className="text-ink-soft">
        <span className="font-medium">{text.adds}: </span>
        {roleNames(t, template, roles).join(", ")}
      </p>
    </div>
  );
}
