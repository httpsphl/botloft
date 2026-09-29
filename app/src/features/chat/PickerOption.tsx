// One choice in a picker below the chat (mode, model): a name, a line
// about what it means for the bot, and a check on the current one.

import { Check, type LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { POPOVER_ITEM } from "../../ui/surface";

export function PickerOption({
  icon: Icon,
  name,
  hint,
  checked,
  danger = false,
  trailing,
  onSelect,
}: {
  icon?: LucideIcon;
  name: string;
  hint: string;
  checked: boolean;
  danger?: boolean;
  trailing?: ReactNode;
  onSelect(): void;
}) {
  return (
    <button
      type="button"
      role="menuitemradio"
      aria-checked={checked}
      onClick={onSelect}
      className={`gap-2.5 py-2 ${POPOVER_ITEM}`}
    >
      {Icon && (
        <Icon
          aria-hidden
          size={16}
          className={`shrink-0 ${danger ? "text-danger" : "text-ink-soft"}`}
        />
      )}
      <span className="min-w-0 flex-1">
        <span className={`block font-medium ${danger ? "text-danger" : "text-ink"}`}>{name}</span>
        <span className="block text-muted text-xs">{hint}</span>
      </span>
      {checked ? <Check aria-hidden size={15} className="shrink-0 text-ink" /> : trailing}
    </button>
  );
}
