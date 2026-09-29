import {
  Archive,
  Ellipsis,
  FolderOpen,
  PanelRight,
  Pause,
  Pencil,
  Play,
  RefreshCcw,
  RotateCw,
  ShieldOff,
} from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "./BotAvatar";
import { BotDialog } from "./BotDialog";
import { BotStateBadge } from "./BotStateBadge";

type Open = "edit" | "archive" | "fresh" | null;

/** The bot's name, state and actions. */
export function BotHeader({
  bot,
  crew,
  detailsOpen,
  onToggleDetails,
}: {
  bot: Bot;
  crew: Crew;
  detailsOpen: boolean;
  onToggleDetails(): void;
}) {
  const t = useT();
  const words = t.bots.header;
  const api = useApi();
  const host = useHost();
  const putBot = useApp((state) => state.putBot);
  const [open, setOpen] = useState<Open>(null);
  const close = () => setOpen(null);
  const stopped = bot.paused || crew.paused;

  const setPaused = (paused: boolean) =>
    attempt(paused ? words.failed.pause : words.failed.resume, async () =>
      putBot(await api.call("bots.setPaused", { botId: bot.id, paused })),
    );
  const restart = (fresh: boolean) =>
    attempt(words.failed.restart, async () =>
      putBot(await api.call("bots.restart", { botId: bot.id, fresh })),
    );

  return (
    <header className="flex items-center gap-3 border-line border-b px-5 py-2.5">
      <BotAvatar color={bot.color} size={36} />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <h1 className="truncate font-semibold text-lg tracking-tight">{bot.name}</h1>
          <span className="font-mono text-muted text-sm" data-selectable>
            @{bot.handle}
          </span>
        </div>
        <div className="mt-0.5 flex items-center gap-3 text-sm">
          <BotStateBadge bot={bot} crewPaused={crew.paused} />
          {bot.permissionMode === "bypass_permissions" && (
            <span
              title={t.chat.mode.badgeHint}
              className="inline-flex shrink-0 items-center gap-1 rounded-full bg-danger/10 px-2 py-0.5 font-medium text-danger text-xs"
            >
              <ShieldOff aria-hidden size={12} />
              {t.chat.mode.badge}
            </span>
          )}
          <span className="truncate text-muted">{bot.role || words.noRole}</span>
        </div>
      </div>
      {bot.paused ? (
        <Button icon={Play} onClick={() => setPaused(false)}>
          {words.resume}
        </Button>
      ) : (
        <Button icon={Pause} onClick={() => setPaused(true)}>
          {words.pause}
        </Button>
      )}
      <Button icon={RotateCw} disabled={stopped} onClick={() => restart(false)}>
        {words.restart}
      </Button>
      <Button
        variant={detailsOpen ? "secondary" : "ghost"}
        icon={PanelRight}
        label={detailsOpen ? words.hideDetails : words.showDetails}
        aria-pressed={detailsOpen}
        onClick={onToggleDetails}
      />
      <Menu
        label={words.more}
        icon={Ellipsis}
        items={[
          { label: words.edit, icon: Pencil, onSelect: () => setOpen("edit") },
          {
            label: words.restartFresh,
            icon: RefreshCcw,
            disabled: stopped,
            onSelect: () => setOpen("fresh"),
          },
          {
            label: words.openFolder,
            icon: FolderOpen,
            onSelect: () => attempt(words.failed.openFolder, () => host.openPath(bot.workspace)),
          },
          { label: words.archive, icon: Archive, danger: true, onSelect: () => setOpen("archive") },
        ]}
      />

      {open === "edit" && <BotDialog bot={bot} onClose={close} />}
      {open === "fresh" && (
        <Confirm
          title={words.fresh.title}
          confirmLabel={words.fresh.confirm}
          onClose={close}
          onConfirm={() => restart(true)}
        >
          {words.fresh.body(bot.name)}
        </Confirm>
      )}
      {open === "archive" && (
        <Confirm
          title={words.archiveConfirm.title(bot.name)}
          confirmLabel={words.archiveConfirm.confirm}
          onClose={close}
          onConfirm={() =>
            attempt(words.failed.archive, async () =>
              putBot(await api.call("bots.archive", { botId: bot.id })),
            )
          }
        >
          {words.archiveConfirm.body}
        </Confirm>
      )}
    </header>
  );
}
