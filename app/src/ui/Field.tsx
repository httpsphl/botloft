import {
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
  type TextareaHTMLAttributes,
  useId,
} from "react";

interface Framing {
  label: string;
  hint?: string | undefined;
  /** Shows "used / max" and warns past it. */
  max?: number | undefined;
  length?: number | undefined;
}

function Frame({
  id,
  label,
  hint,
  max,
  length,
  children,
}: Framing & { id: string; children: ReactNode }) {
  const over = max !== undefined && length !== undefined && length > max;
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-baseline justify-between gap-3">
        <label htmlFor={id} className="font-medium text-ink-soft text-sm">
          {label}
        </label>
        {max !== undefined && length !== undefined && (
          <span className={`font-mono text-xs ${over ? "text-danger" : "text-muted"}`}>
            {length}/{max}
          </span>
        )}
      </div>
      {children}
      {hint && <p className="text-muted text-xs">{hint}</p>}
    </div>
  );
}

const control =
  "w-full rounded-lg border border-line-strong bg-canvas px-2.5 text-ink text-sm outline-none placeholder:text-muted focus:border-accent";

export function TextField({
  label,
  hint,
  max,
  ...input
}: Framing & InputHTMLAttributes<HTMLInputElement>) {
  const id = useId();
  const length = typeof input.value === "string" ? input.value.length : undefined;
  return (
    <Frame id={id} label={label} hint={hint} max={max} length={length}>
      <input id={id} className={`${control} h-8`} spellCheck={false} {...input} />
    </Frame>
  );
}

export function TextArea({
  label,
  hint,
  max,
  ...input
}: Framing & TextareaHTMLAttributes<HTMLTextAreaElement>) {
  const id = useId();
  const length = typeof input.value === "string" ? input.value.length : undefined;
  return (
    <Frame id={id} label={label} hint={hint} max={max} length={length}>
      <textarea id={id} className={`${control} resize-y py-2 leading-relaxed`} {...input} />
    </Frame>
  );
}

export function SelectField({
  label,
  hint,
  options,
  ...select
}: Framing & {
  options: { value: string; label: string }[];
} & SelectHTMLAttributes<HTMLSelectElement>) {
  const id = useId();
  return (
    <Frame id={id} label={label} hint={hint}>
      <select id={id} className={`${control} h-8`} {...select}>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </Frame>
  );
}
