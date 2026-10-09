import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { useApp } from "../../store/context";
import {
  filterCounts,
  STATE_FILTERS,
  type StateFilter,
  setStateFilter,
  useStateFilter,
} from "./stateFilter";

/** Chips above the conversation list: all, needs you, working, unread. */
export function StateFilters() {
  const t = useT();
  const words = t.crews.sidebar.filters;
  const active = useStateFilter();
  const counts = useApp(useShallow(filterCounts));
  const label: Record<StateFilter, string> = {
    all: words.all,
    needs: words.needs,
    working: words.working,
    unread: words.unread,
  };
  return (
    <fieldset
      aria-label={words.label}
      className="m-0 flex min-w-0 flex-wrap gap-1 border-0 p-0 px-2 pb-1.5"
    >
      {STATE_FILTERS.map((filter) => {
        const on = filter === active;
        return (
          <button
            key={filter}
            type="button"
            aria-pressed={on}
            onClick={() => setStateFilter(filter)}
            className={`flex items-center gap-1 rounded-full px-2 py-0.5 font-medium text-xs transition-colors ${on ? "bg-ink text-canvas" : "bg-sunken text-muted hover:text-ink"}`}
          >
            {label[filter]}
            {counts[filter] > 0 && (
              <span className="tabular-nums opacity-70">{counts[filter]}</span>
            )}
          </button>
        );
      })}
    </fieldset>
  );
}
