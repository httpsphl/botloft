// A bot asking to change or delete one of its routines (spec 20.12). A
// change shows what changes, before and after, and the owner can adjust it
// before allowing it; a deletion shows why. Either can be refused with a
// note for the bot. Answered, each shrinks to one line.

import { AlarmClock, Ban, Check, Pencil, TimerOff, Trash2 } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type ApprovalItem, type Bot, FIELD_LIMITS, type Schedule } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";
import { TextArea, TextField } from "../../ui/Field";
import { Switch } from "../../ui/Switch";
import { describeSchedule, zoneLabel } from "./describe";
import { formOf, type RoutineFields, type RoutineForm, scheduleOf } from "./form";
import { MoreOptions } from "./MoreOptions";
import { ScheduleFields } from "./ScheduleFields";

export const CHANGE_ROUTINE_TOOL = "mcp__botloft__change_routine";
export const DELETE_ROUTINE_TOOL = "mcp__botloft__delete_routine";

type Changed = RoutineFields & { routine_id: string; enabled: boolean };

interface Asked {
  name: string;
  before: Changed | null;
  after: Changed | null;
}

function parsed<T>(input: string): Partial<T> {
  try {
    return JSON.parse(input) as Partial<T>;
  } catch {
    return {};
  }
}

/** One thing that changes, before and after, in words. */
function useChanges(before: Changed, after: Changed): [string, string, string][] {
  const t = useT();
  const w = t.chat.routineChange;
  const when = (schedule: Schedule) => describeSchedule(schedule, t.routines.when);
  const onOff = (on: boolean) => (on ? w.on : w.off);
  const rows: [string, string, string][] = [];
  if (before.name !== after.name) rows.push([t.routines.dialog.name, before.name, after.name]);
  if (JSON.stringify(before.schedule) !== JSON.stringify(after.schedule)) {
    rows.push([t.routines.dialog.when, when(before.schedule), when(after.schedule)]);
  }
  if (before.timezone !== after.timezone) {
    rows.push([w.timezone, zoneLabel(before.timezone), zoneLabel(after.timezone)]);
  }
  if (before.enabled !== after.enabled) {
    rows.push([w.state, onOff(before.enabled), onOff(after.enabled)]);
  }
  if (before.overlap !== after.overlap || before.missed !== after.missed) {
    rows.push([w.options, w.optionsBefore, w.optionsAfter]);
  }
  return rows;
}

function answerError(failure: unknown, reasons: Record<string, string>, fallback: string): string {
  const reason = failure instanceof RpcError ? failure.reason : undefined;
  return reason && reason in reasons
    ? (reasons[reason] as string)
    : `${fallback}: ${errorText(failure)}`;
}

export function RoutineChangeCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const w = t.chat.routineChange;
  const d = t.routines.dialog;
  const api = useApi();
  const asked = parsed<Asked>(approval.input);
  const before = asked.before ?? null;
  const after = asked.after ?? null;
  const [form, setForm] = useState<RoutineForm | null>(null);
  const [enabled, setEnabled] = useState(after?.enabled ?? true);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const changes = useChanges(before ?? (after as Changed), after ?? (before as Changed));
  if (approval.status !== "pending" || !before || !after) {
    return <Answered approval={approval} kind="change" />;
  }
  const change = (patch: Partial<RoutineForm>) =>
    setForm((current) => (current ? { ...current, ...patch } : current));

  const answer = async (allow: boolean) => {
    setBusy(true);
    setError(null);
    const adjusted: Changed | null = form
      ? {
          routine_id: after.routine_id,
          name: form.name,
          prompt: form.prompt,
          schedule: scheduleOf(form),
          timezone: form.timezone,
          overlap: form.overlap,
          missed: form.missed,
          enabled,
        }
      : null;
    const trimmed = note.trim();
    try {
      await api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(!allow && trimmed ? { note: trimmed } : {}),
        ...(allow && adjusted ? { input: JSON.stringify(adjusted) } : {}),
      });
    } catch (failure) {
      setError(answerError(failure, d.reasons, allow ? w.applyFailed : w.declineFailed));
    }
    setBusy(false);
  };

  return (
    <SectionCard
      label={w.title(bot.name, asked.name ?? before.name)}
      icon={AlarmClock}
      tone="accent"
      footer={
        <>
          <NoteArea
            label={w.noteLabel(bot.name)}
            value={note}
            onChange={setNote}
            placeholder={w.notePlaceholder(bot.name)}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
              {w.apply}
            </Button>
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {w.decline}
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-3 px-4 py-3 text-sm">
        {changes.length > 0 && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5">
            {changes.map(([what, was, becomes]) => (
              <div key={what} className="contents">
                <dt className="text-muted">{what}</dt>
                <dd>
                  <span className="text-muted line-through">{was}</span>
                  {" → "}
                  <span className="font-medium">{becomes}</span>
                </dd>
              </div>
            ))}
          </dl>
        )}
        {before.prompt !== after.prompt && (
          <details className="text-xs">
            <summary className="cursor-pointer text-muted">{w.newPrompt}</summary>
            <p className="mt-1 whitespace-pre-wrap text-ink-soft" data-selectable>
              {after.prompt}
            </p>
          </details>
        )}
        {form ? (
          <div className="flex flex-col gap-4 border-line border-t pt-3">
            <TextField
              label={d.name}
              value={form.name}
              max={80}
              onChange={(event) => change({ name: event.target.value })}
            />
            <TextArea
              label={d.prompt(bot.name)}
              value={form.prompt}
              max={FIELD_LIMITS.message}
              rows={4}
              onChange={(event) => change({ prompt: event.target.value })}
            />
            <ScheduleFields form={form} change={change} bot={bot.name} />
            <MoreOptions form={form} change={change} />
            <Switch checked={enabled} onChange={setEnabled} label={w.state} />
          </div>
        ) : (
          <Button
            size="sm"
            variant="ghost"
            icon={Pencil}
            className="self-start"
            onClick={() => setForm(formOf(after))}
          >
            {w.adjust}
          </Button>
        )}
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </div>
    </SectionCard>
  );
}

export function RoutineDeleteCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const w = t.chat.routineDelete;
  const api = useApi();
  const asked = parsed<{ name: string; schedule: Schedule; reason: string | null }>(approval.input);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (approval.status !== "pending") {
    return <Answered approval={approval} kind="delete" />;
  }
  const answer = async (allow: boolean) => {
    setBusy(true);
    setError(null);
    const trimmed = note.trim();
    try {
      await api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(!allow && trimmed ? { note: trimmed } : {}),
      });
    } catch (failure) {
      setError(`${w.failed}: ${errorText(failure)}`);
    }
    setBusy(false);
  };
  return (
    <SectionCard
      label={w.title(bot.name, asked.name ?? "")}
      icon={Trash2}
      tone="danger"
      footer={
        <>
          <NoteArea
            label={w.noteLabel(bot.name)}
            value={note}
            onChange={setNote}
            placeholder={w.notePlaceholder(bot.name)}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button variant="danger" icon={Trash2} disabled={busy} onClick={() => answer(true)}>
              {w.delete}
            </Button>
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {w.keep}
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-2 px-4 py-3 text-sm">
        {asked.schedule && (
          <p className="text-ink-soft">{describeSchedule(asked.schedule, t.routines.when)}</p>
        )}
        {asked.reason && (
          <p>
            <span className="text-muted">{w.reason(bot.name)} </span>
            <span data-selectable>{asked.reason}</span>
          </p>
        )}
        <p className="text-muted text-xs">{w.explain}</p>
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </div>
    </SectionCard>
  );
}

function Answered({ approval, kind }: { approval: ApprovalItem; kind: "change" | "delete" }) {
  const t = useT();
  const w = kind === "change" ? t.chat.routineChange : t.chat.routineDelete;
  const name = parsed<{ name: string }>(approval.input).name ?? "";
  const [icon, text, tone] =
    approval.status === "allowed"
      ? ([Check, w.done(name), "ok"] as const)
      : approval.status === "denied"
        ? ([Ban, w.declined(name), "danger"] as const)
        : ([TimerOff, w.expired(name), "quiet"] as const);
  return <SettledLine icon={icon} tone={tone} text={text} note={approval.note} />;
}
