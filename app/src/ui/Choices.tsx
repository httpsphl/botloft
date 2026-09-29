import { useId } from "react";

/**
 * A short set of options where one is chosen, shown side by side. Real
 * radio buttons underneath, so the arrow keys and screen readers work.
 */
export function Choices<T extends string | number>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: readonly { value: T; label: string }[];
  onChange(value: T): void;
}) {
  const name = useId();
  return (
    <div role="radiogroup" aria-label={label} className="flex flex-wrap gap-1.5">
      {options.map((option) => {
        const chosen = option.value === value;
        return (
          <label
            key={String(option.value)}
            className={`flex h-8 cursor-pointer items-center rounded-lg border px-3 text-sm has-[input:focus-visible]:outline-2 has-[input:focus-visible]:outline-accent ${
              chosen
                ? "border-ink bg-ink text-canvas"
                : "border-line-strong bg-panel text-ink hover:bg-sunken"
            }`}
          >
            <input
              type="radio"
              name={name}
              className="sr-only"
              checked={chosen}
              onChange={() => onChange(option.value)}
            />
            {option.label}
          </label>
        );
      })}
    </div>
  );
}
