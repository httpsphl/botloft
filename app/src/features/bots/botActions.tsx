import {
  Archive,
  Crown,
  FolderOpen,
  Mail,
  MailOpen,
  Pencil,
  RefreshCcw,
  Trash2,
} from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp, useHost } from "../../store/context";
import { isUnread } from "../../store/seen";
import { Confirm } from "../../ui/Confirm";
import type { MenuItem } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotDialog } from "./BotDialog";
import { isChief } from "./ChiefBadge";
import { DeleteBot } from "./DeleteBot";

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
  const unread = useApp((state) => isUnread(state, bot.id));
  const markSeen = useApp((state) => state.markSeen);
  const markUnread = useApp((state) => state.markUnread);
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
    unread
      ? { label: words.markRead, icon: MailOpen, onSelect: () => markSeen(bot.id) }
      : { label: words.markUnread, icon: Mail, onSelect: () => markUnread(bot.id) },
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
      <DeleteBot bot={bot} chiefOf={chief ? crew.name : undefined} onClose={close} />
    ) : null;

  return { items, dialogs };
}
