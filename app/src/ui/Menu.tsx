import type { LucideIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "./Button";

export interface MenuItem {
  label: string;
  icon?: LucideIcon;
  danger?: boolean;
  disabled?: boolean;
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

  useEffect(() => {
    if (!open) {
      return;
    }
    const onPointer = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && setOpen(false);
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

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
          className="absolute top-full right-0 z-30 mt-1 min-w-60 border border-line-strong bg-panel py-1"
        >
          {items.map(({ label: itemLabel, icon: Icon, danger, disabled, onSelect }) => (
            <button
              key={itemLabel}
              type="button"
              role="menuitem"
              disabled={disabled}
              onClick={() => {
                setOpen(false);
                onSelect();
              }}
              className={`flex h-8 w-full items-center gap-2 whitespace-nowrap px-3 text-left text-sm hover:bg-sunken disabled:opacity-45 ${danger ? "text-danger" : "text-ink"}`}
            >
              {Icon && <Icon aria-hidden size={14} />}
              {itemLabel}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
