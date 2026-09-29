// The crew's chief suggesting a new bot (spec 10.2). The owner can change
// the name, role, model and instructions before creating it, or say no
// with a note for the chief. Once answered it shrinks to one line.

import { Ban, Check, ChevronRight, TimerOff, UserPlus } from "lucide-react";
import { useId, useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot, BotModel } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { SelectField, TextArea, TextField } from "../../ui/Field";
import { useAnswer } from "./useAnswer";

export const SUGGEST_TOOL = "mcp__botloft__suggest_bot";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

interface Suggestion {
  name: string;
  role: string;
  instructions: string;
  model?: BotModel;
  reason: string;
}

/** The suggestion in the request's input; empty fields if it is cut short. */
export function suggestionOf(input: string): Suggestion {
  try {
    const parsed = JSON.parse(input) as Partial<Suggestion>;
    return {
      name: parsed.name ?? "",
      role: parsed.role ?? "",
      instructions: parsed.instructions ?? "",
      ...(parsed.model && { model: parsed.model }),
      reason: parsed.reason ?? "",
    };
  } catch {
    return { name: "", role: "", instructions: input, reason: "" };
  }
}

export function SuggestionCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const s = t.chat.suggestion;
  const asked = suggestionOf(approval.input);
  const [edited, setEdited] = useState<Suggestion>(asked);
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: s.createFailed,
    deny: s.declineFailed,
  });
  const noteId = useId();
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const changed = JSON.stringify(edited) !== JSON.stringify(asked);
  const set = (field: keyof Suggestion) => (value: string) =>
    setEdited((current) => ({ ...current, [field]: value }));

  return (
    <section
      aria-label={s.title(bot.name)}
      className="my-1 overflow-hidden rounded-2xl border border-line bg-panel shadow-sm"
    >
      <p className="flex items-center gap-2.5 border-line border-b px-4 py-3 font-semibold text-sm">
        <span className="grid h-7 w-7 place-items-center rounded-full bg-accent/12 text-accent">
          <UserPlus aria-hidden size={15} />
        </span>
        {s.title(bot.name)}
      </p>
      <div className="flex flex-col gap-3 px-4 py-3">
        {asked.reason && (
          <p className="text-ink-soft text-sm" data-selectable>
            <span className="font-medium text-ink">{s.why}: </span>
            {asked.reason}
          </p>
        )}
        <div className="grid grid-cols-[1fr_12rem] gap-3">
          <TextField
            label={s.name}
            value={edited.name}
            onChange={(event) => set("name")(event.target.value)}
          />
          <SelectField
            label={s.model}
            value={edited.model ?? "default"}
            onChange={(event) => set("model")(event.target.value)}
            options={MODELS.map((value) => ({ value, label: t.chat.model.names[value] }))}
          />
        </div>
        <TextField
          label={s.role}
          value={edited.role}
          onChange={(event) => set("role")(event.target.value)}
        />
        <TextArea
          label={s.instructions}
          value={edited.instructions}
          rows={5}
          onChange={(event) => set("instructions")(event.target.value)}
        />
        <p className="text-muted text-xs">{s.startsNow(bot.name)}</p>
      </div>
      <div className="border-line border-t bg-canvas/40 px-4 py-3">
        <label htmlFor={noteId} className="sr-only">
          {s.noteLabel(bot.name)}
        </label>
        <textarea
          id={noteId}
          rows={2}
          value={note}
          onChange={(event) => setNote(event.target.value)}
          placeholder={s.notePlaceholder(bot.name)}
          className="block w-full resize-none rounded-xl border border-line-strong bg-panel px-3 py-2 text-sm outline-none placeholder:text-muted focus:border-muted"
        />
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button
            variant="primary"
            icon={Check}
            disabled={busy || !edited.name.trim()}
            onClick={() => answer(true, changed ? JSON.stringify(edited) : undefined)}
          >
            {s.create}
          </Button>
          <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
            {s.decline}
          </Button>
        </div>
      </div>
    </section>
  );
}

/** One line once answered; what was suggested stays a click away. */
function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const s = t.chat.suggestion;
  const suggestion = suggestionOf(approval.input);
  const name = suggestion.name || s.name;
  const [Icon, text, tone] =
    approval.status === "allowed"
      ? [Check, s.created(name), "text-ok"]
      : approval.status === "denied"
        ? [Ban, s.declined(name), "text-danger"]
        : [TimerOff, s.expired(name), "text-quiet"];
  return (
    <details className="group">
      <summary className="flex min-w-0 cursor-default items-center gap-2 rounded-lg px-1.5 py-1 text-sm hover:bg-sunken">
        <Icon aria-hidden size={14} className={`shrink-0 ${tone}`} />
        <span className="shrink-0 font-medium">{text}</span>
        {approval.note && (
          <span className="truncate text-muted text-xs">{`“${approval.note}”`}</span>
        )}
        <ChevronRight
          aria-hidden
          size={13}
          className="shrink-0 text-muted transition-transform group-open:rotate-90"
        />
      </summary>
      <dl
        className="mt-1.5 ml-6 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 rounded-xl border border-line bg-panel px-4 py-3 text-sm"
        data-selectable
      >
        <dt className="text-muted">{s.role}</dt>
        <dd>{suggestion.role}</dd>
        <dt className="text-muted">{s.model}</dt>
        <dd>{t.chat.model.names[suggestion.model ?? "default"]}</dd>
        <dt className="text-muted">{s.instructions}</dt>
        <dd className="whitespace-pre-wrap">{suggestion.instructions}</dd>
      </dl>
    </details>
  );
}
