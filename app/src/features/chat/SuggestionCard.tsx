// The crew's chief suggesting a new bot (spec 10.2). The owner can change
// the name, role, model and instructions before creating it, or say no
// with a note for the chief. Once answered it shrinks to one line.

import { Ban, Check, TimerOff, UserPlus } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot, BotModel } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";
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
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const changed = JSON.stringify(edited) !== JSON.stringify(asked);
  const set = (field: keyof Suggestion) => (value: string) =>
    setEdited((current) => ({ ...current, [field]: value }));

  return (
    <SectionCard
      label={s.title(bot.name)}
      icon={UserPlus}
      tone="accent"
      footer={
        <>
          <NoteArea
            label={s.noteLabel(bot.name)}
            value={note}
            onChange={setNote}
            placeholder={s.notePlaceholder(bot.name)}
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
        </>
      }
    >
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
    </SectionCard>
  );
}

/** One line once answered; what was suggested stays a click away. */
function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const s = t.chat.suggestion;
  const suggestion = suggestionOf(approval.input);
  const name = suggestion.name || s.name;
  const [icon, text, tone] =
    approval.status === "allowed"
      ? ([Check, s.created(name), "ok"] as const)
      : approval.status === "denied"
        ? ([Ban, s.declined(name), "danger"] as const)
        : ([TimerOff, s.expired(name), "quiet"] as const);
  return (
    <SettledLine
      icon={icon}
      tone={tone}
      text={text}
      note={approval.note}
      details={
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
          <dt className="text-muted">{s.role}</dt>
          <dd>{suggestion.role}</dd>
          <dt className="text-muted">{s.model}</dt>
          <dd>{t.chat.model.names[suggestion.model ?? "default"]}</dd>
          <dt className="text-muted">{s.instructions}</dt>
          <dd className="whitespace-pre-wrap">{suggestion.instructions}</dd>
        </dl>
      }
    />
  );
}
