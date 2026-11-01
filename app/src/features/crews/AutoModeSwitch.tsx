import { useId } from "react";
import { useT } from "../../i18n";
import { Switch } from "../../ui/Switch";

/**
 * The crew's automatic mode (spec 7.4): on, every agent of the crew works
 * without asking at each step, the ones it has and the ones that join later.
 * `card` is the form version, with its explanation; the other is the compact
 * one for the crew's header.
 */
export function AutoModeSwitch({
  checked,
  onChange,
  card = false,
  disabled,
}: {
  checked: boolean;
  onChange(checked: boolean): void;
  card?: boolean;
  disabled?: boolean | undefined;
}) {
  const t = useT().crews.autoMode;
  const labelId = useId();
  const hintId = useId();
  if (!card) {
    return (
      <div
        title={t.hint}
        className="flex shrink-0 items-center gap-2 rounded-xl border border-line px-2.5 py-1.5 text-sm"
      >
        {/* In a narrow header the short word shows; the whole label is read aloud. */}
        <span id={labelId} className="sr-only font-medium @2xl:not-sr-only">
          {t.label}
        </span>
        <span aria-hidden className="font-medium @2xl:hidden">
          {t.short}
        </span>
        <Switch
          checked={checked}
          onChange={onChange}
          labelledBy={labelId}
          describedBy={hintId}
          disabled={disabled}
        />
        <span id={hintId} className="sr-only">
          {t.hint}
        </span>
      </div>
    );
  }
  return (
    <div
      className={`flex items-start gap-3 rounded-xl border px-3.5 py-3 ${
        checked ? "border-accent/45 bg-accent/8" : "border-line"
      }`}
    >
      <div className="min-w-0 flex-1">
        <p id={labelId} className="font-medium text-sm">
          {t.label}
        </p>
        <p id={hintId} className="mt-0.5 text-muted text-sm">
          {t.hint}
        </p>
      </div>
      <Switch
        checked={checked}
        onChange={onChange}
        labelledBy={labelId}
        describedBy={hintId}
        disabled={disabled}
        className="mt-0.5"
      />
    </div>
  );
}
