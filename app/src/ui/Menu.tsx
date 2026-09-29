import { Check, type LucideIcon } from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { Button } from "./Button";
import { POPOVER, POPOVER_ITEM } from "./surface";
import { useDismiss } from "./useDismiss";

export interface MenuItem {
  label: string;
  icon?: LucideIcon;
  danger?: boolean;
  disabled?: boolean;
  /** Makes the item one of a set of choices, marked when chosen. */
  checked?: boolean;
  onSelect(): void;
}

/** A button that opens a short list of actions. */
export function Menu({
  label,
  icon,
  items,
}: {
  label: string;
  icon: LucideIcon;
  items: MenuItem[];
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);

  useDismiss(
    open,
    root,
    useCallback(() => setOpen(false), []),
  );

  return (
    <div ref={root} className="relative">
      <Button
        variant="ghost"
        icon={icon}
        label={label}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
      />
      {open && (
        <div
          role="menu"
          className={`absolute top-full right-0 mt-1 min-w-60 origin-top-right ${POPOVER}`}
        >
          {items.map(({ label: itemLabel, icon: Icon, danger, disabled, checked, onSelect }) => (
            <button
              key={itemLabel}
              type="button"
              {...(checked === undefined
                ? { role: "menuitem" }
                : { role: "menuitemradio", "aria-checked": checked })}
              disabled={disabled}
              onClick={() => {
                setOpen(false);
                onSelect();
              }}
              className={`h-8 whitespace-nowrap ${POPOVER_ITEM} ${danger ? "text-danger" : "text-ink"}`}
            >
              {Icon && <Icon aria-hidden size={14} />}
              {checked !== undefined && (
                <Check aria-hidden size={14} className={checked ? "" : "invisible"} />
              )}
              {itemLabel}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
