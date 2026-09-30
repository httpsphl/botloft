import { Archive, Crown, FolderOpen, Pencil, RefreshCcw, Trash2 } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp, useHost } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { KeptFolder } from "../../ui/KeptFolder";
import type { MenuItem } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotDialog } from "./BotDialog";
import { isChief } from "./ChiefBadge";

type Open = "edit" | "archive" | "delete" | "fresh" | null;

/**
 * What the owner can do with a bot from a menu, and the dialogs those
 * actions open (`null` while none is). The header's menu and the right-click
 * menu of the conversation list both come from here, so they stay the same.
 */
export function useBotActions(
  bot: Bot,
  crew: Crew,
): { items: MenuItem[]; dialogs: ReactNode | null } {
  const t = useT();
  const words = t.bots.header;
  const api = useApi();
  const host = useHost();
  const putBot = useApp((state) => state.putBot);
  const putCrew = useApp((state) => state.putCrew);
  const dropBot = useApp((state) => state.dropBot);
  const chief = isChief(bot, crew);
  const [open, setOpen] = useState<Open>(null);
  const close = () => setOpen(null);

  const items: MenuItem[] = [
    { label: words.edit, icon: Pencil, onSelect: () => setOpen("edit") },
    {
      label: chief ? words.stopChief : words.makeChief,
      icon: Crown,
      onSelect: () =>
        attempt(words.failed.chief, async () =>
          putCrew(
            await api.call("crews.setLead", { crewId: crew.id, botId: chief ? null : bot.id }),
          ),
        ),
    },
    {
      label: words.restartFresh,
      icon: RefreshCcw,
      disabled: bot.paused || crew.paused,
      onSelect: () => setOpen("fresh"),
    },
    {
      label: words.openFolder,
      icon: FolderOpen,
      onSelect: () => attempt(words.failed.openFolder, () => host.openPath(bot.workspace)),
    },
    { label: words.archive, icon: Archive, danger: true, onSelect: () => setOpen("archive") },
    { label: words.delete, icon: Trash2, danger: true, onSelect: () => setOpen("delete") },
  ];

  const dialogs =
    open === "edit" ? (
      <BotDialog bot={bot} onClose={close} />
    ) : open === "fresh" ? (
      <Confirm
        title={words.fresh.title}
        confirmLabel={words.fresh.confirm}
        onClose={close}
        onConfirm={() =>
          attempt(words.failed.restart, async () =>
            putBot(await api.call("bots.restart", { botId: bot.id, fresh: true })),
          )
        }
      >
        {words.fresh.body(bot.name)}
      </Confirm>
    ) : open === "archive" ? (
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
    ) : open === "delete" ? (
      <Confirm
        title={words.deleteConfirm.title(bot.name)}
        confirmLabel={words.deleteConfirm.confirm}
        onClose={close}
        onConfirm={() =>
          attempt(words.failed.delete, async () =>
            dropBot((await api.call("bots.delete", { botId: bot.id })).botId),
          )
        }
      >
        <p>
          {words.deleteConfirm.removed(bot.name)}
          {chief && ` ${words.deleteConfirm.chief(crew.name)}`}
        </p>
        <KeptFolder path={bot.workspace}>{words.deleteConfirm.kept(bot.name)}</KeptFolder>
      </Confirm>
    ) : null;

  return { items, dialogs };
}
