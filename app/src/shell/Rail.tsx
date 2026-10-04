// The rail at the far left (spec 15.1): the places of the app as icons,
// home, search, questions and routines, with the owner's account at the
// foot. It stays while the list of crews and bots is hidden, with the
// counts that call the owner.

import { AlarmClock, House, type LucideIcon, MessageCircleQuestion, Search } from "lucide-react";
import { AccountArea } from "../features/account/AccountArea";
import { useT } from "../i18n";
import type { Page } from "../store/app";
import { useApp } from "../store/context";
import { openQuestions } from "../store/questions";
import { CountBadge } from "../ui/Badge";
import { prefs } from "./prefs";

function RailButton({
  icon: Icon,
  label,
  hint = label,
  active,
  count = 0,
  countLabel,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  /** The tooltip, when it says more than the name. */
  hint?: string;
  active: boolean;
  count?: number;
  countLabel?: string;
  onClick(): void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      aria-current={active ? "page" : undefined}
      title={hint}
      onClick={onClick}
      className={`relative grid size-10 place-items-center rounded-xl transition-colors ${
        active ? "bg-sunken text-ink" : "text-muted hover:bg-sunken hover:text-ink"
      }`}
    >
      <Icon aria-hidden size={19} strokeWidth={active ? 2.25 : 1.9} />
      {count > 0 && (
        <span className="absolute -top-1 -right-1">
          <CountBadge tone="warn" count={count} label={countLabel ?? String(count)} />
        </span>
      )}
    </button>
  );
}

export function Rail() {
  const t = useT();
  const words = t.shell.rail;
  const page = useApp((state) => state.page);
  const home = useApp((state) => state.page === null && state.selectedCrewId === null);
  const selectCrew = useApp((state) => state.selectCrew);
  const openPage = useApp((state) => state.openPage);
  const questions = useApp((state) => openQuestions(state).length);
  const go = (to: Exclude<Page, null>) => () => openPage(to);
  return (
    <nav
      aria-label={words.label}
      className="flex w-14 shrink-0 flex-col items-center gap-1 border-line border-r bg-canvas pt-2"
    >
      <RailButton
        icon={House}
        label={words.home}
        active={home}
        onClick={() => {
          // Home brings the list back too, if it was hidden.
          prefs.sidebar.set(true);
          selectCrew(null);
        }}
      />
      <RailButton
        icon={Search}
        label={t.search.label}
        hint={t.search.open}
        active={page === "search"}
        onClick={go("search")}
      />
      <RailButton
        icon={MessageCircleQuestion}
        label={t.questions.box.label}
        hint={t.questions.box.open(questions)}
        active={page === "questions"}
        count={questions}
        countLabel={t.questions.box.open(questions)}
        onClick={go("questions")}
      />
      <RailButton
        icon={AlarmClock}
        label={t.routines.page.title}
        active={page === "routines"}
        onClick={go("routines")}
      />
      <div className="mt-auto w-full">
        <AccountArea />
      </div>
    </nav>
  );
}
