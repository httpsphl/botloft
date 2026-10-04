import {
  Archive,
  Ellipsis,
  Folder,
  FolderInput,
  FolderOpen,
  Pause,
  Pencil,
  Play,
  Plus,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { type Tab, Tabs, tabId } from "../../ui/Tabs";
import { attempt } from "../../ui/toast";
import { BotDialog } from "../bots/BotDialog";
import { CallPills, useLiveCalls } from "../messages/CallPills";
import { Composer } from "../messages/Composer";
import { Timeline } from "../messages/Timeline";
import { CrewRoutines } from "../routines/RoutineList";
import { TaskList } from "../tasks/TaskList";
import { CrewBots } from "./CrewBots";
import { CrewDialog } from "./CrewDialog";
import { DeleteCrew } from "./DeleteCrew";

type Open = "bot" | "rename" | "archive" | "delete" | { move: string } | null;
type Pane = "bots" | "timeline" | "tasks" | "routines";

export function CrewView({ crew }: { crew: Crew }) {
  const t = useT();
  const words = t.crews.view;
  const api = useApi();
  const host = useHost();
  const putCrew = useApp((state) => state.putCrew);
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const calls = useLiveCalls((call) => call.crewId === crew.id, crew.id);
  const [open, setOpen] = useState<Open>(null);
  const [pane, setPane] = useState<Pane>("bots");
  const close = () => setOpen(null);
  const tabs: Tab<Pane>[] = [
    { id: "bots", label: words.tabs.bots },
    { id: "timeline", label: words.tabs.timeline },
    { id: "tasks", label: words.tabs.tasks },
    { id: "routines", label: t.routines.tab },
  ];

  const openFolder = () => attempt(words.failed.openFolder, () => host.openPath(crew.workFolder));
  const pickFolder = () =>
    attempt(words.failed.changeFolder, async () => {
      const folder = await host.pickFolder(t.crews.dialog.pickTitle, crew.workFolder);
      if (folder && folder !== crew.workFolder) {
        setOpen({ move: folder });
      }
    });

  const setPaused = (paused: boolean) =>
    attempt(paused ? words.failed.pause : words.failed.resume, async () =>
      putCrew(await api.call("crews.setPaused", { crewId: crew.id, paused })),
    );

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
            <span aria-hidden>·</span>
            <button
              type="button"
              onClick={openFolder}
              title={words.openFolder}
              className="flex min-w-0 items-center gap-1 rounded-lg px-1 hover:bg-sunken hover:text-ink"
            >
              <Folder aria-hidden size={13} className="shrink-0" />
              <span className="truncate">{words.folder(crew.workFolder)}</span>
            </button>
          </p>
        </div>
        <CallPills calls={calls} className="max-w-[45%] justify-end" />
        <Button variant="primary" icon={Plus} onClick={() => setOpen("bot")}>
          {t.crews.newBot}
        </Button>
        {crew.paused ? (
          <Button icon={Play} onClick={() => setPaused(false)}>
            {words.resume}
          </Button>
        ) : (
          <Button icon={Pause} onClick={() => setPaused(true)}>
            {words.pause}
          </Button>
        )}
        <Menu
          label={words.moreActions}
          icon={Ellipsis}
          items={[
            { label: t.crews.rename, icon: Pencil, onSelect: () => setOpen("rename") },
            { label: words.openFolder, icon: FolderOpen, onSelect: openFolder },
            { label: words.changeFolder, icon: FolderInput, onSelect: pickFolder },
            {
              label: words.archive,
              icon: Archive,
              danger: true,
              onSelect: () => setOpen("archive"),
            },
            { label: words.delete, icon: Trash2, danger: true, onSelect: () => setOpen("delete") },
          ]}
        />
      </header>

      <Tabs<Pane> label={words.tabs.label} tabs={tabs} value={pane} onChange={setPane} />
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

      {open === "bot" && <BotDialog crewId={crew.id} onClose={close} />}
      {open === "rename" && <CrewDialog crew={crew} onClose={close} />}
      {typeof open === "object" && open !== null && (
        <Confirm
          title={words.moveTitle(crew.name)}
          confirmLabel={words.move}
          onClose={close}
          onConfirm={() =>
            attempt(words.failed.changeFolder, async () =>
              putCrew(
                await api.call("crews.setWorkFolder", { crewId: crew.id, workFolder: open.move }),
              ),
            )
          }
        >
          {words.moveBody(open.move)}
        </Confirm>
      )}
      {open === "archive" && (
        <Confirm
          title={words.archiveTitle(crew.name)}
          confirmLabel={words.archive}
          onClose={close}
          onConfirm={() =>
            attempt(words.failed.archive, async () =>
              putCrew(await api.call("crews.archive", { crewId: crew.id })),
            )
          }
        >
          {words.archiveBody(bots.length)}
        </Confirm>
      )}
      {open === "delete" && <DeleteCrew crew={crew} bots={bots.length} onClose={close} />}
    </section>
  );
}
