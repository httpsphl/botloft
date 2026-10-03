// "More options" in the routine form (spec 20.9): the time zone, what to
// do when the last run has not finished or the computer was off, and a
// cron expression for those who want one. A routine that waits for a
// signal (spec 20.13) has no time: only the overlap rule is left.

import { ChevronRight } from "lucide-react";
import { useT } from "../../i18n";
import { Choices } from "../../ui/Choices";
import { SelectField, TextField } from "../../ui/Field";
import { allZones, zoneLabel } from "./describe";
import type { RoutineForm } from "./form";

export function MoreOptions({
  form,
  change,
}: {
  form: RoutineForm;
  change(patch: Partial<RoutineForm>): void;
}) {
  const d = useT().routines.dialog;
  const zones = allZones();
  const timed = form.frequency !== "signal" || form.useCron;
  return (
    <details className="group rounded-xl border border-line">
      <summary className="flex cursor-default items-center gap-1.5 px-3 py-2 font-medium text-ink-soft text-sm hover:text-ink">
        <ChevronRight aria-hidden size={14} className="transition-transform group-open:rotate-90" />
        {d.more}
      </summary>
      <div className="flex flex-col gap-4 border-line border-t px-3 py-3">
        {timed && (
          <SelectField
            label={d.timezone}
            value={form.timezone}
            onChange={(event) => change({ timezone: event.target.value })}
            options={zones.map((zone) => ({ value: zone, label: zoneLabel(zone) }))}
          />
        )}
        <div className="flex flex-col gap-1.5">
          <p className="font-medium text-ink-soft text-sm">{d.overlap}</p>
          <Choices
            label={d.overlap}
            value={form.overlap}
            options={[
              { value: "skip", label: d.overlapSkip },
              { value: "queue", label: d.overlapQueue },
            ]}
            onChange={(overlap) => change({ overlap })}
          />
        </div>
        {timed && (
          <div className="flex flex-col gap-1.5">
            <p className="font-medium text-ink-soft text-sm">{d.missed}</p>
            <Choices
              label={d.missed}
              value={form.missed}
              options={[
                { value: "run_once", label: d.missedRunOnce },
                { value: "skip", label: d.missedSkip },
              ]}
              onChange={(missed) => change({ missed })}
            />
            <p className="text-muted text-xs">{d.wakeNote}</p>
          </div>
        )}
        {timed && (
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={form.useCron}
              onChange={(event) => change({ useCron: event.target.checked })}
            />
            {d.advanced}
          </label>
        )}
        {form.useCron && (
          <TextField
            label={d.cron}
            value={form.cron}
            onChange={(event) => change({ cron: event.target.value })}
            placeholder="0 9 * * 1-5"
            hint={d.cronHint}
          />
        )}
      </div>
    </details>
  );
}
