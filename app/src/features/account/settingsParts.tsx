// The pieces the Settings sections share: a titled section, a labelled
// field and an on/off row with its explanation.

import { type ReactNode, useId } from "react";
import { Switch } from "../../ui/Switch";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h3 className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">{title}</h3>
      {children}
    </section>
  );
}

export function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <p className="font-medium text-sm">{label}</p>
      {children}
      {hint && <p className="text-muted text-xs">{hint}</p>}
    </div>
  );
}

/** A setting that is on or off; the hint says what the current choice does. */
export function Toggle({
  label,
  hint,
  checked,
  disabled,
  onChange,
}: {
  label: string;
  hint: string;
  checked: boolean;
  disabled?: boolean | undefined;
  onChange(checked: boolean): void;
}) {
  const id = useId();
  return (
    <div className="flex items-start gap-4">
      <div className="min-w-0 flex-1">
        <label id={`${id}-label`} htmlFor={id} className="font-medium text-sm">
          {label}
        </label>
        <p id={`${id}-hint`} className="mt-0.5 text-muted text-xs leading-relaxed">
          {hint}
        </p>
      </div>
      <Switch
        id={id}
        checked={checked}
        disabled={disabled}
        labelledBy={`${id}-label`}
        describedBy={`${id}-hint`}
        onChange={onChange}
        className="mt-0.5"
      />
    </div>
  );
}
