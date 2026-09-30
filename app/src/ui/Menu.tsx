import { Check, type LucideIcon } from "lucide-react";
import { type KeyboardEvent, useCallback, useRef, useState } from "react";
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

/** The arrow keys, Home and End walk the items of the menu they are pressed in. */
export function menuKeys(event: KeyboardEvent<HTMLElement>): void {
  const items = [
    ...event.currentTarget.querySelectorAll<HTMLElement>('[role^="menuitem"]:not(:disabled)'),
  ];
  const last = items.length - 1;
  const at = items.indexOf(document.activeElement as HTMLElement);
  let next: number;
  switch (event.key) {
    case "ArrowDown":
      next = at >= last ? 0 : at + 1;
      break;
    case "ArrowUp":
      next = at <= 0 ? last : at - 1;
      break;
    case "Home":
      next = 0;
      break;
    case "End":
      next = last;
      break;
    default:
      return;
  }
  event.preventDefault();
  items[next]?.focus();
}

/** The rows of a menu. `onPick` runs first: it closes the menu. */
export function MenuItems({ items, onPick }: { items: MenuItem[]; onPick(): void }) {
  return items.map(({ label, icon: Icon, danger, disabled, checked, onSelect }) => (
    <button
      key={label}
      type="button"
      {...(checked === undefined
        ? { role: "menuitem" }
        : { role: "menuitemradio", "aria-checked": checked })}
      disabled={disabled}
      onClick={() => {
        onPick();
        onSelect();
      }}
      className={`h-8 whitespace-nowrap ${POPOVER_ITEM} ${danger ? "text-danger" : "text-ink"}`}
    >
      {Icon && <Icon aria-hidden size={14} />}
      {checked !== undefined && (
        <Check aria-hidden size={14} className={checked ? "" : "invisible"} />
      )}
      {label}
    </button>
  ));
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
          onKeyDown={menuKeys}
          className={`absolute top-full right-0 mt-1 min-w-60 origin-top-right ${POPOVER}`}
        >
          <MenuItems items={items} onPick={() => setOpen(false)} />
        </div>
      )}
    </div>
  );
}
