import { Archive, Ellipsis, FolderOpen, Pause, Pencil, Play, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { BotDialog } from "../bots/BotDialog";
import { BotStateBadge } from "../bots/BotStateBadge";
import { CrewDialog } from "./CrewDialog";

/** The crew's `shared` folder, next to its bots' workspaces (spec 5). */
export function sharedFolder(bots: Bot[]): string | null {
  const workspace = bots[0]?.workspace;
  return workspace ? workspace.replace(/[\\/][^\\/]+$/, "\\shared") : null;
}

type Open = "bot" | "rename" | "archive" | null;

export function CrewView({ crew }: { crew: Crew }) {
  const api = useApi();
  const host = useHost();
  const putCrew = useApp((state) => state.putCrew);
  const selectBot = useApp((state) => state.selectBot);
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const [open, setOpen] = useState<Open>(null);
  const shared = sharedFolder(bots);
  const close = () => setOpen(null);

  const setPaused = (paused: boolean) =>
    attempt(paused ? "Could not pause the crew" : "Could not resume the crew", async () =>
      putCrew(await api.call("crews.setPaused", { crewId: crew.id, paused })),
    );

  return (
    <section aria-label={crew.name} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-3 border-line border-b px-6 py-4">
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

      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        {bots.length === 0 ? (
          <div className="border border-line border-dashed px-6 py-10 text-center">
            <p className="font-medium">No bots in {crew.name} yet.</p>
            <p className="mt-1 text-muted text-sm">
              A bot is a Claude Code session that keeps running, with its own folder and role.
            </p>
            <Button className="mt-4" variant="primary" icon={Plus} onClick={() => setOpen("bot")}>
              New bot
            </Button>
          </div>
        ) : (
          <ul className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3">
            {bots.map((bot) => (
              <li key={bot.id}>
                <button
                  type="button"
                  onClick={() => selectBot(bot.id)}
                  className="flex h-full w-full flex-col gap-2 border border-line bg-panel p-3 text-left hover:border-line-strong"
                >
                  <div className="flex w-full items-center gap-2.5">
                    <BotAvatar color={bot.color} size={32} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-semibold">{bot.name}</p>
                      <p className="truncate font-mono text-muted text-xs">@{bot.handle}</p>
                    </div>
                  </div>
                  <p className="line-clamp-2 min-h-10 text-ink-soft text-sm">
                    {bot.role || "No role yet."}
                  </p>
                  <BotStateBadge bot={bot} crewPaused={crew.paused} />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

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
