// The pages at the top of the sidebar: the search (spec 8.8) and the
// question box (spec 23.6).

import { MessageCircleQuestion, Search } from "lucide-react";
import { useT } from "../../i18n";
import { useApp } from "../../store/context";
import { openQuestions } from "../../store/questions";

/** A row of the sidebar, as the conversations are. */
export const row =
  "relative flex w-full items-center rounded-lg text-left transition-colors duration-150 hover:bg-sunken";

/** The search (spec 8.8). */
export function SearchEntry() {
  const t = useT();
  const open = useApp((state) => state.page === "search");
  const openPage = useApp((state) => state.openPage);
  return (
    <div className="shrink-0 px-2">
      <button
        type="button"
        aria-current={open ? "page" : undefined}
        title={t.search.open}
        onClick={() => openPage("search")}
        className={`${row} gap-2.5 px-2.5 py-1.5 text-sm ${open ? "bg-sunken text-ink" : "text-ink-soft"}`}
      >
        <Search aria-hidden size={16} className="text-muted" />
        <span className="min-w-0 flex-1 truncate font-medium">{t.search.label}</span>
        <kbd className="font-sans text-[11px] text-muted">Ctrl K</kbd>
      </button>
    </div>
  );
}

/** The question box (spec 23.6), with how many questions wait. */
export function QuestionsEntry() {
  const t = useT();
  const words = t.questions.box;
  const count = useApp((state) => openQuestions(state).length);
  const open = useApp((state) => state.page === "questions");
  const openPage = useApp((state) => state.openPage);
  return (
    <div className="shrink-0 px-2">
      <button
        type="button"
        aria-current={open ? "page" : undefined}
        title={words.open(count)}
        onClick={() => openPage("questions")}
        className={`${row} gap-2.5 px-2.5 py-1.5 text-sm ${open ? "bg-sunken text-ink" : "text-ink-soft"}`}
      >
        <MessageCircleQuestion
          aria-hidden
          size={16}
          className={count > 0 ? "text-warn" : "text-muted"}
        />
        <span className="min-w-0 flex-1 truncate font-medium">{words.label}</span>
        {count > 0 && (
          <span className="grid h-[18px] min-w-[18px] place-items-center rounded-full bg-warn px-1 font-semibold text-[11px] text-canvas tabular-nums">
            <span aria-hidden>{count}</span>
            <span className="sr-only">{words.open(count)}</span>
          </span>
        )}
      </button>
    </div>
  );
}
