import { Check, ChevronDown } from "lucide-react";
import { type KeyboardEvent, useCallback, useEffect, useId, useRef, useState } from "react";
import { POPOVER, POPOVER_ITEM } from "./surface";
import { useDismiss } from "./useDismiss";

/**
 * A choice among several, shown as a button with the chosen one that
 * opens the list under it, like the app's menus. The arrow keys move
 * through the list, Enter picks, and Escape closes only the list, never
 * the dialog around it.
 */
export function Select<T extends string>({
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
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const id = useId();
  const chosen = options.find((option) => option.value === value);
  const close = useCallback(() => setOpen(false), []);
  useDismiss(open, root, close);

  // The list opens on the chosen option, ready for the keyboard.
  useEffect(() => {
    if (open) {
      list.current?.querySelector<HTMLElement>('[aria-selected="true"]')?.focus();
    }
  }, [open]);

  const pick = (next: T) => {
    setOpen(false);
    trigger.current?.focus();
    if (next !== value) {
      onChange(next);
    }
  };

  const onListKey = (event: KeyboardEvent<HTMLDivElement>) => {
    const items = [...(list.current?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])];
    const at = items.indexOf(document.activeElement as HTMLElement);
    const moves: Record<string, number> = {
      ArrowDown: Math.min(items.length - 1, at + 1),
      ArrowUp: Math.max(0, at - 1),
      Home: 0,
      End: items.length - 1,
    };
    if (event.key in moves) {
      event.preventDefault();
      items[moves[event.key] ?? 0]?.focus();
    } else if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      setOpen(false);
      trigger.current?.focus();
    } else if (event.key === "Tab") {
      setOpen(false);
    }
  };

  return (
    <div ref={root} className="relative w-full max-w-72">
      <button
        ref={trigger}
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={open ? `${id}-list` : undefined}
        aria-label={`${label}: ${chosen?.label ?? ""}`}
        onClick={() => setOpen(!open)}
        onKeyDown={(event) => {
          if (event.key === "ArrowDown" || event.key === "ArrowUp") {
            event.preventDefault();
            setOpen(true);
          }
        }}
        className="flex h-8 w-full items-center justify-between gap-2 rounded-lg border border-line-strong bg-panel px-3 text-left text-sm hover:bg-sunken"
      >
        <span className="truncate">{chosen?.label}</span>
        <ChevronDown
          aria-hidden
          size={14}
          className={`shrink-0 text-muted transition-transform duration-150 ${open ? "rotate-180" : ""}`}
        />
      </button>
      {open && (
        <div
          ref={list}
          id={`${id}-list`}
          role="listbox"
          aria-label={label}
          tabIndex={-1}
          onKeyDown={onListKey}
          className={`absolute top-full left-0 mt-1 w-full origin-top ${POPOVER}`}
        >
          {options.map((option) => {
            const selected = option.value === value;
            return (
              <div
                key={option.value}
                role="option"
                aria-selected={selected}
                tabIndex={-1}
                onClick={() => pick(option.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    pick(option.value);
                  }
                }}
                className={`h-8 cursor-pointer focus:bg-sunken focus:outline-none ${POPOVER_ITEM}`}
              >
                <Check aria-hidden size={14} className={selected ? "" : "invisible"} />
                <span className="truncate">{option.label}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
