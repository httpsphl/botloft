import { Archive, Ellipsis, FolderOpen, Pause, Pencil, Play, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { attempt } from "../../ui/toast";
import { BotDialog } from "../bots/BotDialog";
import { Composer } from "../messages/Composer";
import { Timeline } from "../messages/Timeline";
import { TaskList } from "../tasks/TaskList";
import { CrewBots } from "./CrewBots";
import { CrewDialog } from "./CrewDialog";

/** The crew's `shared` folder, next to its bots' workspaces (spec 5). */
export function sharedFolder(bots: Bot[]): string | null {
  const workspace = bots[0]?.workspace;
  return workspace ? workspace.replace(/[\\/][^\\/]+$/, "\\shared") : null;
}

type Open = "bot" | "rename" | "archive" | null;
type Pane = "bots" | "timeline" | "tasks";

const TABS: Tab<Pane>[] = [
  { id: "bots", label: "Bots" },
  { id: "timeline", label: "Timeline" },
  { id: "tasks", label: "Tasks" },
];

export function CrewView({ crew }: { crew: Crew }) {
  const api = useApi();
  const host = useHost();
  const putCrew = useApp((state) => state.putCrew);
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const [open, setOpen] = useState<Open>(null);
  const [pane, setPane] = useState<Pane>("bots");
  const shared = sharedFolder(bots);
  const close = () => setOpen(null);

  const setPaused = (paused: boolean) =>
    attempt(paused ? "Could not pause the crew" : "Could not resume the crew", async () =>
      putCrew(await api.call("crews.setPaused", { crewId: crew.id, paused })),
    );

  return (
    <section aria-label={crew.name} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-3 border-line border-b px-5 py-3.5">
        <div className="min-w-0 flex-1">
          <h1 className="truncate font-semibold text-xl tracking-tight">{crew.name}</h1>
          <p className="mt-0.5 text-muted text-sm">
            {bots.length === 1 ? "1 bot" : `${bots.length} bots`}
            {crew.paused && " · paused: its bots stay stopped until you resume it"}
          </p>
        </div>
        <Button variant="primary" icon={Plus} onClick={() => setOpen("bot")}>
          New bot
        </Button>
        {crew.paused ? (
          <Button icon={Play} onClick={() => setPaused(false)}>
            Resume crew
          </Button>
        ) : (
          <Button icon={Pause} onClick={() => setPaused(true)}>
            Pause crew
          </Button>
        )}
        <Menu
          label="More crew actions"
          icon={Ellipsis}
          items={[
            { label: "Rename", icon: Pencil, onSelect: () => setOpen("rename") },
            {
              label: "Open shared folder",
              icon: FolderOpen,
              disabled: !shared,
              onSelect: () =>
                shared && attempt("Could not open the folder", () => host.openPath(shared)),
            },
            {
              label: "Archive crew",
              icon: Archive,
              danger: true,
              onSelect: () => setOpen("archive"),
            },
          ]}
        />
      </header>

      <Tabs<Pane> label="Crew views" tabs={TABS} value={pane} onChange={setPane} />
      {pane === "bots" && (
        <div
          role="tabpanel"
          aria-labelledby={tabId("bots")}
          className="min-h-0 flex-1 overflow-y-auto p-5"
        >
          <CrewBots crew={crew} bots={bots} onNewBot={() => setOpen("bot")} />
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
            empty="No messages yet. Write to a bot below; what the bots send each other shows up here too."
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

      {open === "bot" && <BotDialog crewId={crew.id} onClose={close} />}
      {open === "rename" && <CrewDialog crew={crew} onClose={close} />}
      {open === "archive" && (
        <Confirm
          title={`Archive ${crew.name}?`}
          confirmLabel="Archive crew"
          onClose={close}
          onConfirm={() =>
            attempt("Could not archive the crew", async () =>
              putCrew(await api.call("crews.archive", { crewId: crew.id })),
            )
          }
        >
          The crew leaves the app.{" "}
          {bots.length > 0 &&
            `${bots.length === 1 ? "Its bot stops" : `Its ${bots.length} bots stop`}, and messages still waiting for them are not delivered.`}
        </Confirm>
      )}
    </section>
  );
}
