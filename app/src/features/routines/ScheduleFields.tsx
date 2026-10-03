// "When" in the routine form: how often, the days and the time, or the
// interval. A cron expression, under "More options", takes its place.

import { useId } from "react";
import { useT } from "../../i18n";
import { Choices } from "../../ui/Choices";
import { weekdayName } from "./describe";
import type { Frequency, RoutineForm, Unit } from "./form";

const control =
  "h-8 rounded-lg border border-line-strong bg-canvas px-2.5 text-ink text-sm outline-none focus:border-accent";

export function ScheduleFields({
  form,
  change,
  bot,
}: {
  form: RoutineForm;
  change(patch: Partial<RoutineForm>): void;
  /** The bot's name, for the signal's hint. */
  bot: string;
}) {
  const d = useT().routines.dialog;
  const timeId = useId();
  const everyId = useId();
  const signalId = useId();
  const frequencies: { value: Frequency; label: string }[] = [
    { value: "daily", label: d.frequency.daily },
    { value: "weekdays", label: d.frequency.weekdays },
    { value: "days", label: d.frequency.days },
    { value: "interval", label: d.frequency.interval },
    { value: "signal", label: d.frequency.signal },
  ];

  const toggleDay = (day: number) =>
    change({
      days: form.days.includes(day) ? form.days.filter((d) => d !== day) : [...form.days, day],
    });

  return (
    <fieldset className="flex flex-col gap-2.5" disabled={form.useCron}>
      <legend className="mb-1 font-medium text-ink-soft text-sm">{d.when}</legend>
      <Choices
        label={d.when}
        value={form.frequency}
        options={frequencies}
        onChange={(frequency) => change({ frequency })}
      />
      {form.frequency === "days" && (
        <fieldset aria-label={d.frequency.days} className="flex gap-1.5">
          {[1, 2, 3, 4, 5, 6, 7].map((day) => {
            const on = form.days.includes(day);
            return (
              <button
                key={day}
                type="button"
                aria-pressed={on}
                aria-label={weekdayName(day)}
                title={weekdayName(day)}
                onClick={() => toggleDay(day)}
                className={`grid h-8 w-10 place-items-center rounded-lg border text-sm transition-colors ${
                  on
                    ? "border-accent bg-accent/12 font-medium text-accent-text"
                    : "border-line-strong text-ink-soft hover:bg-sunken"
                }`}
              >
                {weekdayName(day, "short")}
              </button>
            );
          })}
        </fieldset>
      )}
      {form.frequency === "signal" ? (
        <div className="flex flex-col gap-1.5">
          <div className="flex items-center gap-2">
            <label htmlFor={signalId} className="text-ink-soft text-sm">
              {d.signal}
            </label>
            <input
              id={signalId}
              required
              value={form.signal}
              placeholder={d.signalPlaceholder}
              onChange={(event) => change({ signal: event.target.value })}
              className={`${control} w-56`}
            />
          </div>
          <p className="text-muted text-xs">{d.signalHint(bot)}</p>
        </div>
      ) : form.frequency === "interval" ? (
        <div className="flex items-center gap-2">
          <label htmlFor={everyId} className="text-ink-soft text-sm">
            {d.every}
          </label>
          <input
            id={everyId}
            type="number"
            min={1}
            value={form.every}
            onChange={(event) => change({ every: Number(event.target.value) })}
            className={`${control} w-20`}
          />
          <select
            aria-label={d.every}
            value={form.unit}
            onChange={(event) => change({ unit: event.target.value as Unit })}
            className={control}
          >
            <option value="minutes">{d.minutes}</option>
            <option value="hours">{d.hours}</option>
          </select>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <label htmlFor={timeId} className="text-ink-soft text-sm">
            {d.at}
          </label>
          <input
            id={timeId}
            type="time"
            required
            value={form.time}
            onChange={(event) => change({ time: event.target.value })}
            className={`${control} w-32`}
          />
        </div>
      )}
    </fieldset>
  );
}
