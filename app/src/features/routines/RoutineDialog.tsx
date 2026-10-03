// Creating a routine for a bot, or editing one (spec 20.9): its name,
// what the bot should do, and when. A refusal the daemon explains with a
// reason is worded here, in the owner's language.

import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type Bot, FIELD_LIMITS, type Routine } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { TextArea, TextField } from "../../ui/Field";
import { blankForm, formOf, type RoutineForm, scheduleOf } from "./form";
import { MoreOptions } from "./MoreOptions";
import { ScheduleFields } from "./ScheduleFields";

type Props = { bot: Bot; onClose(): void } & ({ routine?: undefined } | { routine: Routine });

export function RoutineDialog({ bot, routine, onClose }: Props) {
  const t = useT();
  const d = t.routines.dialog;
  const api = useApi();
  const putRoutine = useApp((state) => state.putRoutine);
  const [form, setForm] = useState<RoutineForm>(() => (routine ? formOf(routine) : blankForm()));
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const change = (patch: Partial<RoutineForm>) => setForm((current) => ({ ...current, ...patch }));

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    const fields = {
      name: form.name,
      prompt: form.prompt,
      schedule: scheduleOf(form),
      timezone: form.timezone,
      overlap: form.overlap,
      missed: form.missed,
    };
    try {
      putRoutine(
        routine
          ? await api.call("routines.update", { routineId: routine.id, ...fields })
          : await api.call("routines.create", { botId: bot.id, ...fields }),
      );
      onClose();
    } catch (failure) {
      const reason = failure instanceof RpcError ? failure.reason : undefined;
      const worded =
        reason && reason in d.reasons ? d.reasons[reason as keyof typeof d.reasons] : null;
      setError(worded ?? errorText(failure));
      setBusy(false);
    }
  };

  const formId = "routine-form";
  return (
    <Dialog
      title={routine ? d.editTitle(routine.name) : d.newTitle(bot.name)}
      onClose={onClose}
      width="lg"
      footer={
        <>
          <Button onClick={onClose}>{t.common.cancel}</Button>
          <Button variant="primary" type="submit" form={formId} disabled={busy}>
            {routine ? d.save : d.create}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-4">
        <TextField
          label={d.name}
          value={form.name}
          max={80}
          onChange={(event) => change({ name: event.target.value })}
          placeholder={d.namePlaceholder}
          autoFocus
          required
        />
        <TextArea
          label={d.prompt(bot.name)}
          value={form.prompt}
          max={FIELD_LIMITS.message}
          rows={4}
          onChange={(event) => change({ prompt: event.target.value })}
          placeholder={d.promptPlaceholder}
          required
        />
        <ScheduleFields form={form} change={change} bot={bot.name} />
        <MoreOptions form={form} change={change} />
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
