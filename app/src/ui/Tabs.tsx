import { useLayoutEffect, useRef, useState } from "react";

export interface Tab<T extends string> {
  id: T;
  label: string;
}

/**
 * A row of tabs; the panels are the caller's, labelled by `tabId(id)`. The
 * accent line under the selected tab slides to the next one.
 */
export function Tabs<T extends string>({
  label,
  tabs,
  value,
  onChange,
}: {
  label: string;
  tabs: Tab<T>[];
  value: T;
  onChange(value: T): void;
}) {
  const list = useRef<HTMLDivElement>(null);
  const [line, setLine] = useState<{ left: number; width: number } | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: re-measured when the selection or the tabs change
  useLayoutEffect(() => {
    const selected = list.current?.querySelector<HTMLElement>('[aria-selected="true"]');
    if (selected) {
      setLine({ left: selected.offsetLeft + 4, width: selected.offsetWidth - 8 });
    }
  }, [value, tabs]);

  return (
    <div
      ref={list}
      role="tablist"
      aria-label={label}
      className="relative flex h-9 shrink-0 gap-1 border-line border-b px-3"
    >
      {tabs.map((tab) => {
        const selected = tab.id === value;
        return (
          <button
            key={tab.id}
            id={tabId(tab.id)}
            type="button"
            role="tab"
            aria-selected={selected}
            onClick={() => onChange(tab.id)}
            className={`px-2.5 font-medium text-sm transition-colors ${selected ? "text-ink" : "text-muted hover:text-ink"}`}
          >
            {tab.label}
          </button>
        );
      })}
      {line && (
        <span
          aria-hidden
          className="absolute bottom-0 h-0.5 rounded-full bg-accent transition-[left,width] duration-300 ease-[var(--ease-out)]"
          style={{ left: line.left, width: line.width }}
        />
      )}
    </div>
  );
}

export function tabId(id: string): string {
  return `tab-${id}`;
}
