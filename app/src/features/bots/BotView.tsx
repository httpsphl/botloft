import {
  Archive,
  Ellipsis,
  FolderOpen,
  Pause,
  Pencil,
  Play,
  RefreshCcw,
  RotateCw,
} from "lucide-react";
import { useState } from "react";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "./BotAvatar";
import { BotDialog } from "./BotDialog";
import { BotStateBadge, stateView } from "./BotStateBadge";

type Open = "edit" | "archive" | "fresh" | null;

export function BotView({ bot, crew }: { bot: Bot; crew: Crew }) {
  const api = useApi();
  const host = useHost();
  const putBot = useApp((state) => state.putBot);
  const [open, setOpen] = useState<Open>(null);
  const close = () => setOpen(null);
  const stopped = bot.paused || crew.paused;
  const view = stateView(bot, crew.paused);

  const setPaused = (paused: boolean) =>
    attempt(paused ? "Could not pause the bot" : "Could not resume the bot", async () =>
      putBot(await api.call("bots.setPaused", { botId: bot.id, paused })),
    );
  const restart = (fresh: boolean) =>
    attempt("Could not restart the bot", async () =>
      putBot(await api.call("bots.restart", { botId: bot.id, fresh })),
    );

  return (
    <section aria-label={bot.name} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-3 border-line border-b px-6 py-3.5">
        <BotAvatar color={bot.color} size={40} />
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <h1 className="truncate font-semibold text-xl tracking-tight">{bot.name}</h1>
            <span className="font-mono text-muted text-sm" data-selectable>
              @{bot.handle}
            </span>
          </div>
          <div className="mt-0.5 flex items-center gap-3 text-sm">
            <BotStateBadge bot={bot} crewPaused={crew.paused} />
            <span className="truncate text-muted">{bot.role || "No role"}</span>
          </div>
        </div>
        {bot.paused ? (
          <Button icon={Play} onClick={() => setPaused(false)}>
            Resume
          </Button>
        ) : (
          <Button icon={Pause} onClick={() => setPaused(true)}>
            Pause
          </Button>
        )}
        <Button icon={RotateCw} disabled={stopped} onClick={() => restart(false)}>
          Restart
        </Button>
        <Menu
          label="More bot actions"
          icon={Ellipsis}
          items={[
            { label: "Edit", icon: Pencil, onSelect: () => setOpen("edit") },
            {
              label: "Restart with a new conversation",
              icon: RefreshCcw,
              disabled: stopped,
              onSelect: () => setOpen("fresh"),
            },
            {
              label: "Open folder",
              icon: FolderOpen,
              onSelect: () =>
                attempt("Could not open the folder", () => host.openPath(bot.workspace)),
            },
            {
              label: "Archive bot",
              icon: Archive,
              danger: true,
              onSelect: () => setOpen("archive"),
            },
          ]}
        />
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto p-6">
        <div className="flex max-w-3xl flex-col gap-4">
          {crew.paused && !bot.paused && (
            <Callout title={`${crew.name} is paused`}>
              Its bots stay stopped until you resume the crew.
            </Callout>
          )}
          {(view.tone === "warn" || view.tone === "danger") && (
            <Callout tone={view.tone} title={view.label}>
              {view.hint}
            </Callout>
          )}
          <dl className="grid grid-cols-[9rem_1fr] gap-x-4 gap-y-3 border border-line bg-panel p-4 text-sm">
            <dt className="text-muted">Folder</dt>
            <dd className="break-all font-mono text-xs" data-selectable>
              {bot.workspace}
            </dd>
            <dt className="text-muted">Process</dt>
            <dd className="font-mono text-xs">
              {bot.generation === null ? "not started" : `generation ${bot.generation}`}
            </dd>
            <dt className="text-muted">Instructions</dt>
            <dd className="whitespace-pre-wrap text-ink-soft" data-selectable>
              {bot.instructions || "None yet."}
            </dd>
          </dl>
        </div>
      </div>

      {open === "edit" && <BotDialog bot={bot} onClose={close} />}
      {open === "fresh" && (
        <Confirm
          title="Start a new conversation?"
          confirmLabel="Restart"
          onClose={close}
          onConfirm={() => restart(true)}
        >
          {bot.name} restarts without its current conversation. Its folder and its CLAUDE.md stay as
          they are.
        </Confirm>
      )}
      {open === "archive" && (
        <Confirm
          title={`Archive ${bot.name}?`}
          confirmLabel="Archive bot"
          onClose={close}
          onConfirm={() =>
            attempt("Could not archive the bot", async () =>
              putBot(await api.call("bots.archive", { botId: bot.id })),
            )
          }
        >
          The bot stops and leaves the crew. Messages still waiting for it are not delivered.
        </Confirm>
      )}
    </section>
  );
}
