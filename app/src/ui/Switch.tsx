/**
 * An on/off switch: a button with `role="switch"`, so screen readers say
 * whether it is on. Name it with `label` or `labelledBy`.
 */
export function Switch({
  checked,
  onChange,
  id,
  label,
  labelledBy,
  describedBy,
  disabled,
  className = "",
}: {
  checked: boolean;
  onChange(checked: boolean): void;
  id?: string;
  label?: string;
  labelledBy?: string;
  describedBy?: string;
  disabled?: boolean | undefined;
  className?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      id={id}
      aria-checked={checked}
      aria-label={label}
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative h-5 w-9 shrink-0 rounded-full transition-colors duration-200 disabled:opacity-45 ${
        checked ? "bg-accent" : "bg-line-strong"
      } ${className}`}
    >
      <span
        className={`absolute top-0.5 left-0.5 h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-200 ${
          checked ? "translate-x-4" : ""
        }`}
      />
    </button>
  );
}
