// The form in an agent's card (spec 8.11): its fields and one send button.
// The daemon checks the values again; here a required field keeps the
// button off until it is filled in.

import { type FormEvent, useId, useState } from "react";
import { useT } from "../../i18n";
import type { CardField } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";

const INPUT =
  "w-full rounded-xl border border-line-strong bg-panel px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted focus:border-muted disabled:opacity-60";

export function CardForm({
  fields,
  submit,
  disabled,
  send,
}: {
  fields: CardField[];
  submit?: string | undefined;
  disabled: boolean;
  send: (values: Record<string, string>) => void;
}) {
  const t = useT().agentCards;
  const [values, setValues] = useState<Record<string, string>>({});
  const set = (id: string, value: string) => setValues((now) => ({ ...now, [id]: value }));
  const missing = fields.some(
    (field) => field.required && field.kind !== "check" && !(values[field.id] ?? "").trim(),
  );
  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (!missing && !disabled) {
      send(
        Object.fromEntries(
          fields.map((field) => [
            field.id,
            values[field.id] ?? (field.kind === "check" ? "false" : ""),
          ]),
        ),
      );
    }
  };
  return (
    <form onSubmit={onSubmit} className="flex flex-col gap-3">
      {fields.map((field) => (
        <Field
          key={field.id}
          field={field}
          value={values[field.id] ?? ""}
          disabled={disabled}
          set={(value) => set(field.id, value)}
        />
      ))}
      <div>
        <Button type="submit" variant="primary" disabled={disabled || missing}>
          {submit ?? t.send}
        </Button>
      </div>
    </form>
  );
}

function Field({
  field,
  value,
  disabled,
  set,
}: {
  field: CardField;
  value: string;
  disabled: boolean;
  set: (value: string) => void;
}) {
  const t = useT().agentCards;
  const id = useId();
  if (field.kind === "check") {
    return (
      <label htmlFor={id} className="flex items-center gap-2 text-sm">
        <input
          id={id}
          type="checkbox"
          checked={value === "true"}
          disabled={disabled}
          onChange={(event) => set(event.target.checked ? "true" : "false")}
          className="h-4 w-4 accent-[var(--accent)]"
        />
        {field.label}
      </label>
    );
  }
  const head = (
    <label htmlFor={id} className="text-sm">
      {field.label}
      {field.required && <span className="ml-1.5 text-muted text-xs">{t.required}</span>}
    </label>
  );
  return (
    <div className="flex flex-col gap-1">
      {head}
      {field.kind === "choice" ? (
        <select
          id={id}
          value={value}
          disabled={disabled}
          required={field.required}
          onChange={(event) => set(event.target.value)}
          className={INPUT}
        >
          <option value="">{t.pick}</option>
          {field.options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      ) : field.kind === "long_text" ? (
        <textarea
          id={id}
          rows={3}
          value={value}
          disabled={disabled}
          required={field.required}
          onChange={(event) => set(event.target.value)}
          className={`resize-y ${INPUT}`}
        />
      ) : (
        <input
          id={id}
          type="text"
          inputMode={field.kind === "number" ? "decimal" : undefined}
          value={value}
          disabled={disabled}
          required={field.required}
          onChange={(event) => set(event.target.value)}
          className={INPUT}
        />
      )}
    </div>
  );
}
