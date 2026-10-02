import { Pause, Plus } from "lucide-react";
import { memo, useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { useShallow } from "zustand/react/shallow";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Activity, Bot, Crew } from "../../lib/protocol.gen";
import { activityOf, botsOf, crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { hasUnreadReply } from "../../store/seen";
import { Button } from "../../ui/Button";
import { ContextMenu, menuPoint, type Point } from "../../ui/ContextMenu";
import { APP_OPENED } from "../../ui/motion";
import { AccountArea } from "../account/AccountArea";
import { ListAvatar } from "../bots/BotAvatar";
import { BotStateBadge, stateView } from "../bots/BotStateBadge";
import { useBotActions } from "../bots/botActions";
import { ChiefBadge, isChief } from "../bots/ChiefBadge";
import { toolAction, toolTitle } from "../chat/toolNames";
import { CrewDialog } from "./CrewDialog";

/** Crews as sections and their bots as conversations (spec 15.1). */
export function Sidebar() {
  const t = useT();
  const crews = useApp(useShallow(crewList));
  const [creating, setCreating] = useState(false);
  return (
    <div className="flex w-72 shrink-0 flex-col border-line border-r bg-panel">
      <nav aria-label={t.crews.sidebar.label} className="flex min-h-0 flex-1 flex-col">
        <div className="flex h-10 shrink-0 items-center justify-between pr-1.5 pl-4">
          <h2 className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">
            {t.crews.sidebar.label}
          </h2>
          <Button
            variant="ghost"
            size="sm"
            icon={Plus}
            label={t.crews.newCrew}
            onClick={() => setCreating(true)}
          />
        </div>
        <ul className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
          {crews.map((crew) => (
            <CrewEntry key={crew.id} crew={crew} />
          ))}
        </ul>
      </nav>
      <AccountArea />
      {creating && <CrewDialog onClose={() => setCreating(false)} />}
    </div>
  );
}

const row =
  "relative flex w-full items-center rounded-lg text-left transition-colors duration-150 hover:bg-sunken";

function CrewEntry({ crew }: { crew: Crew }) {
  const t = useT();
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const selected = useApp((state) => state.selectedCrewId === crew.id && !state.selectedBotId);
  const selectCrew = useApp((state) => state.selectCrew);

  return (
    <li className="mt-2">
      <button
        type="button"
        aria-current={selected ? "page" : undefined}
        onClick={() => selectCrew(crew.id)}
        className={`${row} h-8 gap-2 px-2.5 font-semibold text-sm ${selected ? "bg-sunken" : ""}`}
      >
        <span className="min-w-0 flex-1 truncate">{crew.name}</span>
        {crew.paused && (
          <span className="flex items-center gap-1 font-medium text-quiet text-xs">
            <Pause aria-hidden size={12} />
            {t.crews.paused}
          </span>
        )}
      </button>
      <ul>
        {bots.map((bot) => (
          <Conversation key={bot.id} bot={bot} crew={crew} />
        ))}
      </ul>
    </li>
  );
}

/** Memoized: a change to one bot re-renders only its own line. */
const Conversation = memo(function Conversation({ bot, crew }: { bot: Bot; crew: Crew }) {
  const t = useT();
  const selected = useApp((state) => state.selectedBotId === bot.id);
  const selectBot = useApp((state) => state.selectBot);
  // A right-click opens the bot's menu, the same as in its header.
  const actions = useBotActions(bot, crew);
  const [menuAt, setMenuAt] = useState<Point | null>(null);
  const closeMenu = useCallback(() => setMenuAt(null), []);
  const activity = useApp((state) => activityOf(state, bot.id));
  const unread = useApp((state) => hasUnreadReply(state, bot.id));
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
                className={`shrink-0 text-xs ${unread ? "font-semibold text-accent" : "text-muted"}`}
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

/** The conversation-list line, worded here from what the daemon sends. */
/** The line in the owner's language: the daemon sends no wording. */
function activityText(activity: Activity, t: Messages): string {
  const { text, tool } = activity;
  switch (activity.kind) {
    case "owner":
      return t.crews.sidebar.fromOwner(text);
    case "approval":
      return t.crews.sidebar.awaitingApproval(tool ? toolAction(tool, t.tools) : text);
    case "tool": {
      if (!tool) {
        return text;
      }
      const title = toolTitle(tool, t.tools);
      return text ? `${title} · ${text}` : title;
    }
    default:
      return text;
  }
}
