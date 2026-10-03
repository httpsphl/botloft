// A bot asking for a routine (spec 20.12), for itself or for another bot
// of its crew. The owner can change the name, what to do and when before
// creating it, or say no with a note for the bot. A refusal the daemon
// explains with a reason is worded here. Once answered it shrinks to one
// line.

import { AlarmClock, Ban, Check, ChevronRight, TimerOff } from "lucide-react";
import { useId, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type ApprovalItem, type Bot, FIELD_LIMITS } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { TextArea, TextField } from "../../ui/Field";
import { describeSchedule, systemZone } from "./describe";
import { formOf, type RoutineFields, type RoutineForm, scheduleOf } from "./form";
import { MoreOptions } from "./MoreOptions";
import { ScheduleFields } from "./ScheduleFields";

export const ROUTINE_TOOL = "mcp__botloft__schedule_routine";

/** The routine asked for, and the bot it is for by handle. */
interface Asked {
  routine: RoutineFields;
  bot?: string;
}

/** The routine in the request's input; defaults for what is cut short. */
export function askedOf(input: string): Asked {
  const blank: RoutineFields = {
    name: "",
    prompt: "",
    schedule: { kind: "weekly", days: [1, 2, 3, 4, 5], time: "09:00" },
    timezone: systemZone(),
    overlap: "skip",
    missed: "run_once",
  };
  try {
    const { bot, ...fields } = JSON.parse(input) as Partial<RoutineFields> & { bot?: string };
    return { routine: { ...blank, ...fields }, ...(bot && { bot }) };
  } catch {
    return { routine: { ...blank, prompt: input } };
  }
}

function formFields(form: RoutineForm): RoutineFields {
  return {
    name: form.name,
    prompt: form.prompt,
    schedule: scheduleOf(form),
    timezone: form.timezone,
    overlap: form.overlap,
    missed: form.missed,
  };
}

/** The bot the routine is for: the one that asked, or one of its crew. */
function useRunner(bot: Bot, handle: string | undefined): Bot {
  const bots = useApp((state) => state.bots);
  if (!handle) {
    return bot;
  }
  const found = Object.values(bots).find(
    (other) => other.crewId === bot.crewId && other.handle === handle,
  );
  return found ?? { ...bot, name: `@${handle}` };
}

export function RoutineRequestCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const r = t.chat.routineRequest;
  const d = t.routines.dialog;
  const api = useApi();
  const asked = askedOf(approval.input);
  const runner = useRunner(bot, asked.bot);
  const [form, setForm] = useState<RoutineForm>(() => formOf(asked.routine));
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const noteId = useId();
  if (approval.status !== "pending") {
    return <Answered approval={approval} />;
  }
  const change = (patch: Partial<RoutineForm>) => setForm((current) => ({ ...current, ...patch }));

  const answer = async (allow: boolean) => {
    setBusy(true);
    setError(null);
    const edited = formFields(form);
    const changed = JSON.stringify(edited) !== JSON.stringify(formFields(formOf(asked.routine)));
    const trimmed = note.trim();
    try {
      await api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(!allow && trimmed ? { note: trimmed } : {}),
        ...(allow && changed ? { input: JSON.stringify(edited) } : {}),
      });
    } catch (failure) {
      const reason = failure instanceof RpcError ? failure.reason : undefined;
      const worded =
        reason && reason in d.reasons ? d.reasons[reason as keyof typeof d.reasons] : null;
      setError(worded ?? `${allow ? r.createFailed : r.declineFailed}: ${errorText(failure)}`);
    }
    setBusy(false);
  };

  const title = asked.bot ? r.titleFor(bot.name, runner.name) : r.title(bot.name);
  return (
    <section
      aria-label={title}
      className="my-1 overflow-hidden rounded-2xl border border-line bg-panel shadow-sm animate-attention"
    >
      <p className="flex items-center gap-2.5 border-line border-b px-4 py-3 font-semibold text-sm">
        <span className="grid h-7 w-7 place-items-center rounded-full bg-accent/12 text-accent">
          <AlarmClock aria-hidden size={15} />
        </span>
        {title}
      </p>
      <div className="flex flex-col gap-4 px-4 py-3">
        <TextField
          label={d.name}
          value={form.name}
          max={80}
          onChange={(event) => change({ name: event.target.value })}
        />
        <TextArea
          label={d.prompt(runner.name)}
          value={form.prompt}
          max={FIELD_LIMITS.message}
          rows={4}
          onChange={(event) => change({ prompt: event.target.value })}
        />
        <ScheduleFields form={form} change={change} bot={runner.name} />
        <MoreOptions form={form} change={change} />
        <p className="text-muted text-xs">{r.explain(runner.name)}</p>
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </div>
      <div className="border-line border-t bg-canvas/40 px-4 py-3">
        <label htmlFor={noteId} className="sr-only">
          {r.noteLabel(bot.name)}
        </label>
        <textarea
          id={noteId}
          rows={2}
          value={note}
          onChange={(event) => setNote(event.target.value)}
          placeholder={r.notePlaceholder(bot.name)}
          className="block w-full resize-none rounded-xl border border-line-strong bg-panel px-3 py-2 text-sm outline-none placeholder:text-muted focus:border-muted"
        />
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button
            variant="primary"
            icon={Check}
            disabled={busy || !form.name.trim() || !form.prompt.trim()}
            onClick={() => answer(true)}
          >
            {r.create}
          </Button>
          <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
            {r.decline}
          </Button>
        </div>
      </div>
    </section>
  );
}

/** One line once answered; what was asked stays a click away. */
function Answered({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const r = t.chat.routineRequest;
  const { routine } = askedOf(approval.input);
  const name = routine.name || t.routines.dialog.name;
  const [Icon, text, tone] =
    approval.status === "allowed"
      ? [Check, r.created(name), "text-ok"]
      : approval.status === "denied"
        ? [Ban, r.declined(name), "text-danger"]
        : [TimerOff, r.expired(name), "text-quiet"];
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
        <dt className="text-muted">{t.routines.dialog.when}</dt>
        <dd>{describeSchedule(routine.schedule, t.routines.when)}</dd>
        <dt className="text-muted">{t.routines.dialog.name}</dt>
        <dd>{routine.name}</dd>
        <dd className="col-span-2 whitespace-pre-wrap">{routine.prompt}</dd>
      </dl>
    </details>
  );
}
