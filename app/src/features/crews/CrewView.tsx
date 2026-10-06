import { Ellipsis, Folder, Plus, Sparkles } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Menu } from "../../ui/Menu";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { CallPills, useLiveCalls } from "../messages/CallPills";
import { Composer } from "../messages/Composer";
import { Timeline } from "../messages/Timeline";
import { CrewRoutines } from "../routines/RoutineList";
import { TaskList } from "../tasks/TaskList";
import { CrewBots } from "./CrewBots";
import { useCrewActions } from "./crewActions";

type Pane = "bots" | "timeline" | "tasks" | "routines";

export function CrewView({ crew }: { crew: Crew }) {
  const t = useT();
  const words = t.crews.view;
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const calls = useLiveCalls((call) => call.crewId === crew.id, crew.id);
  const actions = useCrewActions(crew);
  const [pane, setPane] = useState<Pane>("bots");
  const tabs: Tab<Pane>[] = [
    { id: "bots", label: words.tabs.bots },
    { id: "timeline", label: words.tabs.timeline },
    { id: "tasks", label: words.tabs.tasks },
    { id: "routines", label: t.routines.tab },
  ];

  return (
    <section aria-label={crew.name} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-3 border-line border-b px-5 py-3.5">
        <div className="min-w-0 flex-1">
          <h1 className="truncate font-semibold text-xl tracking-tight">{crew.name}</h1>
          <p className="mt-0.5 flex min-w-0 items-center gap-1.5 text-muted text-sm">
            <span className="shrink-0">
              {words.bots(bots.length)}
              {crew.paused && ` · ${words.pausedNote}`}
            </span>
            {/* Only the icon: the path is in its tooltip, and a click opens it. */}
            <button
              type="button"
              onClick={actions.openFolder}
              title={`${words.folder(crew.workFolder)}
${words.openFolderHint}`}
              aria-label={`${words.openFolder}: ${crew.workFolder}`}
              className="grid size-6 shrink-0 place-items-center rounded-lg hover:bg-sunken hover:text-ink"
            >
              <Folder aria-hidden size={14} />
            </button>
          </p>
        </div>
        <CallPills calls={calls} className="max-w-[45%] justify-end" />
        <Button icon={Sparkles} title={t.catalog.openHint} onClick={actions.agency.onSelect}>
          {actions.agency.label}
        </Button>
        <Button variant="primary" icon={Plus} onClick={actions.newBot.onSelect}>
          {actions.newBot.label}
        </Button>
        <Button icon={actions.pause.icon} onClick={actions.pause.onSelect}>
          {actions.pause.label}
        </Button>
        <Menu label={words.moreActions} icon={Ellipsis} items={actions.items} />
      </header>

      <Tabs<Pane> label={words.tabs.label} tabs={tabs} value={pane} onChange={setPane} />
      {pane === "bots" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("bots")}
          className="min-h-0 flex-1 overflow-y-auto p-5"
        >
          <CrewBots crew={crew} bots={bots} onNewBot={actions.newBot.onSelect} />
        </div>
      )}
      {pane === "timeline" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("timeline")}
          className="flex min-h-0 flex-1 flex-col"
        >
          <Timeline
            filter={{ crewId: crew.id }}
            empty={words.timelineEmpty}
            composer={(onSent) => <Composer crewId={crew.id} onSent={onSent} />}
          />
        </div>
      )}
      {pane === "tasks" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("tasks")}
          className="flex min-h-0 flex-1 flex-col"
        >
          <TaskList crewId={crew.id} />
        </div>
      )}

      {pane === "routines" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("routines")}
          className="flex min-h-0 flex-1 flex-col"
        >
          <CrewRoutines crew={crew} />
        </div>
      )}

      {actions.dialogs}
    </section>
  );
}
