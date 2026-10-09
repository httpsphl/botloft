import { memo, useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import { plainText } from "../../lib/plainText";
import type { Activity, Bot, Crew } from "../../lib/protocol.gen";
import { activityOf } from "../../store/app";
import { useApp } from "../../store/context";
import { isUnread } from "../../store/seen";
import { ContextMenu, menuPoint, type Point } from "../../ui/ContextMenu";
import { APP_OPENED } from "../../ui/motion";
import { BotStateBadge, stateView } from "../bots/BotStateBadge";
import { useBotActions } from "../bots/botActions";
import { ChiefBadge, isChief } from "../bots/ChiefBadge";
import { ListAvatar } from "../bots/ListAvatar";
import { toolAction, toolTitle } from "../chat/toolNames";
import { row } from "./SidebarPages";

/** Memoized: a change to one bot re-renders only its own line. */
export const Conversation = memo(function Conversation({ bot, crew }: { bot: Bot; crew: Crew }) {
  const t = useT();
  const selected = useApp((state) => state.selectedBotId === bot.id);
  const selectBot = useApp((state) => state.selectBot);
  // A right-click opens the bot's menu, the same as in its header.
  const actions = useBotActions(bot, crew);
  const [menuAt, setMenuAt] = useState<Point | null>(null);
  const closeMenu = useCallback(() => setMenuAt(null), []);
  const activity = useApp((state) => activityOf(state, bot.id));
  const unread = useApp((state) => isUnread(state, bot.id));
  return (
    <li className={bot.createdAt > APP_OPENED ? "animate-rise" : undefined}>
      <button
        type="button"
        aria-current={selected ? "page" : undefined}
        aria-label={[bot.name, stateView(bot, crew.paused).label, unread && t.crews.sidebar.unread]
          .filter(Boolean)
          .join(", ")}
        onClick={() => selectBot(bot.id)}
        onContextMenu={(event) => {
          event.preventDefault();
          setMenuAt(menuPoint(event));
        }}
        className={`${row} gap-2.5 px-2.5 py-2 ${selected || menuAt ? "bg-sunken" : ""}`}
      >
        <ListAvatar bot={bot} crewPaused={crew.paused} size={32} />
        <span className="min-w-0 flex-1">
          <span className="flex items-baseline gap-2">
            <span className="flex min-w-0 flex-1 items-center gap-1">
              <span className={`truncate text-sm ${unread ? "font-semibold" : "font-medium"}`}>
                {bot.name}
              </span>
              {isChief(bot, crew) && <ChiefBadge crew={crew} compact />}
            </span>
            {activity && (
              <time
                className={`shrink-0 text-xs ${unread ? "font-semibold text-accent-text" : "text-muted"}`}
                dateTime={new Date(activity.at).toISOString()}
              >
                {when(activity.at)}
              </time>
            )}
          </span>
          <span className="mt-0.5 flex min-w-0 items-center gap-1.5 text-xs">
            <BotStateBadge bot={bot} crewPaused={crew.paused} compact />
            <span className={`min-w-0 flex-1 truncate ${unread ? "text-ink" : "text-muted"}`}>
              {activity ? activityText(activity, t) : bot.role || t.crews.sidebar.noMessages}
            </span>
            {unread && <span aria-hidden className="size-2.5 shrink-0 rounded-full bg-accent" />}
          </span>
        </span>
      </button>
      {menuAt && (
        <ContextMenu
          label={t.bots.header.menuOf(bot.name)}
          items={actions.items}
          at={menuAt}
          onClose={closeMenu}
        />
      )}
      {/* Outside the list: a dialog is not part of the navigation. */}
      {actions.dialogs && createPortal(actions.dialogs, document.body)}
    </li>
  );
});

/**
 * The conversation-list line, worded here in the owner's language from what
 * the daemon sends (it sends no wording). What someone wrote shows as plain
 * words: the chat renders its Markdown, a line would show the marks raw. A
 * command or a path keeps its own characters.
 */
function activityText(activity: Activity, t: Messages): string {
  const { text, tool } = activity;
  switch (activity.kind) {
    case "owner":
      return t.crews.sidebar.fromOwner(plainText(text));
    case "approval":
      return t.crews.sidebar.awaitingApproval(tool ? toolAction(tool, t.tools) : text);
    case "question":
      return t.questions.activity(plainText(text));
    case "tool": {
      if (!tool) {
        return text;
      }
      const title = toolTitle(tool, t.tools);
      return text ? `${title} · ${text}` : title;
    }
    default:
      return plainText(text);
  }
}
