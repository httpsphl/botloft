import { Ellipsis, Folder, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApp } from "../../store/context";
import { Button, FitLabel } from "../../ui/Button";
import { Menu } from "../../ui/Menu";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { CallPills, useLiveCalls } from "../messages/CallPills";
import { Composer } from "../messages/Composer";
import { Timeline } from "../messages/Timeline";
import { CrewRoutines } from "../routines/RoutineList";
import { TaskList } from "../tasks/TaskList";
import { AutoModeSwitch } from "./AutoModeSwitch";
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
      {/* The name keeps its room: when the buttons do not fit beside it, they
          go to a line of their own under it instead of covering it. */}
      <header className="@container flex flex-wrap items-center gap-x-3 gap-y-2.5 border-line border-b px-5 py-3.5">
        <div className="min-w-48 flex-1">
          <h1 className="truncate font-semibold text-xl tracking-tight">{crew.name}</h1>
          <p className="mt-0.5 flex min-w-0 items-center gap-1.5 text-muted text-sm">
            <span className="truncate">
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
        <div className="flex flex-wrap items-center gap-2">
          <AutoModeSwitch checked={crew.autoMode} onChange={actions.setAutoMode} />
          <Button title={t.catalog.openHint} onClick={actions.agency.onSelect}>
            <span className="sr-only @2xl:not-sr-only">{actions.agency.label}</span>
            <span aria-hidden className="@2xl:hidden">
              {t.catalog.short}
            </span>
          </Button>
          <Button variant="primary" icon={Plus} onClick={actions.newBot.onSelect}>
            {actions.newBot.label}
          </Button>
          <Button
            icon={actions.pause.icon}
            title={actions.pause.label}
            onClick={actions.pause.onSelect}
          >
            <FitLabel>{actions.pause.label}</FitLabel>
          </Button>
          <Menu label={words.moreActions} icon={Ellipsis} items={actions.items} />
        </div>
      </header>

      <div className="relative">
        <Tabs<Pane> label={words.tabs.label} tabs={tabs} value={pane} onChange={setPane} />
        {/* Calls between the crew's bots, at the end of the tabs' row, where
            there is room and nothing moves when one comes. */}
        <CallPills
          calls={calls}
          className="absolute top-1/2 right-5 max-h-8 max-w-[55%] -translate-y-1/2 justify-end overflow-hidden"
        />
      </div>
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
