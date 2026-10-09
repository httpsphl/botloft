import { Check } from "lucide-react";
import { useT } from "../../i18n";
import { AVATAR_PALETTE } from "../../lib/protocol.gen";

/** Swatches for a crew's color, and a way to have none. */
export function CrewColorField({
  value,
  onChange,
}: {
  value: string | null;
  onChange(color: string | null): void;
}) {
  const words = useT().crews.dialog;
  const same = (a: string | null, b: string | null) => a?.toLowerCase() === b?.toLowerCase();
  return (
    <fieldset className="m-0 flex flex-col gap-1.5 border-0 p-0">
      <legend className="mb-1.5 font-medium text-ink-soft text-sm">{words.color}</legend>
      <div className="flex flex-wrap items-center gap-1.5">
        <button
          type="button"
          aria-pressed={value === null}
          onClick={() => onChange(null)}
          className={`rounded-lg px-2 py-1 font-medium text-xs ${value === null ? "bg-sunken text-ink ring-2 ring-accent" : "text-muted hover:text-ink"}`}
        >
          {words.noColor}
        </button>
        {AVATAR_PALETTE.map((swatch) => {
          const on = same(value, swatch);
          return (
            <button
              key={swatch}
              type="button"
              aria-label={words.swatch(swatch)}
              aria-pressed={on}
              onClick={() => onChange(swatch)}
              className={`relative grid size-7 place-items-center rounded-full ${on ? "ring-2 ring-accent ring-offset-2 ring-offset-panel" : ""}`}
              style={{ background: swatch }}
            >
              {on && <Check aria-hidden size={14} className="text-black/70" />}
            </button>
          );
        })}
      </div>
    </fieldset>
  );
}
