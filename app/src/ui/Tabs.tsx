export interface Tab<T extends string> {
  id: T;
  label: string;
}

/** A row of tabs; the panels are the caller's, labelled by `tabId(id)`. */
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
  return (
    <div
      role="tablist"
      aria-label={label}
      className="flex h-9 shrink-0 gap-1 border-line border-b px-3"
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
            className={`relative px-2.5 font-medium text-sm ${selected ? "text-ink after:absolute after:inset-x-1 after:bottom-0 after:h-0.5 after:bg-accent" : "text-muted hover:text-ink"}`}
          >
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}

export function tabId(id: string): string {
  return `tab-${id}`;
}
