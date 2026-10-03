import {
  ChevronDown,
  ChevronsDownUp,
  ChevronsUpDown,
  MessageCircleQuestion,
  Pause,
  Plus,
} from "lucide-react";
import { memo, useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { useShallow } from "zustand/react/shallow";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Activity, Bot, Crew } from "../../lib/protocol.gen";
import { activityOf, botsOf, crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { openQuestions } from "../../store/questions";
import { isUnread, unreadIn } from "../../store/seen";
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
import { setAllCollapsed, setCollapsed, useCollapsed, useCollapsedSet } from "./collapsed";

/** Crews as sections and their bots as conversations (spec 15.1). */
export function Sidebar() {
  const t = useT();
  const words = t.crews.sidebar;
  const crews = useApp(useShallow(crewList));
  const overview = useApp((state) => state.selectedCrewId === null && !state.questionBox);
  const selectCrew = useApp((state) => state.selectCrew);
  const collapsed = useCollapsedSet();
  const allFolded = crews.length > 0 && crews.every((crew) => collapsed.has(crew.id));
  const [creating, setCreating] = useState(false);
  return (
    <div className="flex w-72 shrink-0 flex-col border-line border-r bg-panel">
      <nav aria-label={words.label} className="flex min-h-0 flex-1 flex-col">
        <div className="flex h-10 shrink-0 items-center gap-0.5 pr-1.5 pl-2">
          <h2 className="min-w-0 flex-1">
            <button
              type="button"
              aria-current={overview ? "page" : undefined}
              title={words.showAll}
              onClick={() => selectCrew(null)}
              className={`rounded-md px-2 py-1 font-semibold text-xs uppercase tracking-[0.12em] transition-colors hover:bg-sunken hover:text-ink ${overview ? "bg-sunken text-ink" : "text-muted"}`}
            >
              {words.label}
            </button>
          </h2>
          <Button
            variant="ghost"
            size="sm"
            icon={allFolded ? ChevronsUpDown : ChevronsDownUp}
            label={allFolded ? words.expandAll : words.collapseAll}
            onClick={() => setAllCollapsed(allFolded ? [] : crews.map((crew) => crew.id))}
          />
          <Button
            variant="ghost"
            size="sm"
            icon={Plus}
            label={t.crews.newCrew}
            onClick={() => setCreating(true)}
          />
        </div>
        <QuestionsEntry />
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

/** The question box (spec 23.6), with how many questions wait. */
function QuestionsEntry() {
  const t = useT();
  const words = t.questions.box;
  const count = useApp((state) => openQuestions(state).length);
  const open = useApp((state) => state.questionBox);
  const openBox = useApp((state) => state.openQuestionBox);
  return (
    <div className="shrink-0 px-2">
      <button
        type="button"
        aria-current={open ? "page" : undefined}
        title={words.open(count)}
        onClick={openBox}
        className={`${row} gap-2.5 px-2.5 py-1.5 text-sm ${open ? "bg-sunken text-ink" : "text-ink-soft"}`}
      >
        <MessageCircleQuestion
          aria-hidden
          size={16}
          className={count > 0 ? "text-warn" : "text-muted"}
        />
        <span className="min-w-0 flex-1 truncate font-medium">{words.label}</span>
        {count > 0 && (
          <span className="grid h-[18px] min-w-[18px] place-items-center rounded-full bg-warn px-1 font-semibold text-[11px] text-canvas tabular-nums">
            <span aria-hidden>{count}</span>
            <span className="sr-only">{words.open(count)}</span>
          </span>
        )}
      </button>
    </div>
  );
}

function CrewEntry({ crew }: { crew: Crew }) {
  const t = useT();
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const selected = useApp((state) => state.selectedCrewId === crew.id && !state.selectedBotId);
  const unread = useApp((state) => unreadIn(state, crew.id));
  const selectCrew = useApp((state) => state.selectCrew);
  const openBotId = useApp((state) => state.selectedBotId);
  const collapsed = useCollapsed(crew.id);
  // Folded, the open conversation still shows, and a dot tells a bot waits.
  const shown = collapsed ? bots.filter((bot) => bot.id === openBotId) : bots;
  const waiting =
    collapsed && bots.some((bot) => bot.state === "needs_approval" || bot.state === "auth_error");
  const words = t.crews.sidebar;

  return (
    <li className="mt-2">
      <div className="relative flex items-center">
        <button
          type="button"
          aria-expanded={!collapsed}
          aria-label={collapsed ? words.expand(crew.name) : words.collapse(crew.name)}
          title={collapsed ? words.expand(crew.name) : words.collapse(crew.name)}
          onClick={() => setCollapsed(crew.id, !collapsed)}
          className="absolute left-1 z-10 grid size-6 place-items-center rounded-md text-muted hover:bg-line hover:text-ink"
        >
          <ChevronDown
            aria-hidden
            size={14}
            className={`transition-transform duration-150 ${collapsed ? "-rotate-90" : ""}`}
          />
        </button>
        <button
          type="button"
          aria-current={selected ? "page" : undefined}
          onClick={() => selectCrew(crew.id)}
          className={`${row} h-8 gap-2 pr-2.5 pl-8 font-semibold text-sm ${selected ? "bg-sunken" : ""}`}
        >
          <span className="min-w-0 flex-1 truncate">{crew.name}</span>
          {waiting && (
            <span className="flex shrink-0">
              <span aria-hidden className="live-dot" style={{ background: "var(--warn)" }} />
              <span className="sr-only">{words.waiting}</span>
            </span>
          )}
          {unread > 0 && (
            <span className="grid h-[18px] min-w-[18px] place-items-center rounded-full bg-accent px-1 font-semibold text-[11px] text-canvas tabular-nums">
              <span aria-hidden>{unread}</span>
              <span className="sr-only">{t.crews.sidebar.unreadCount(unread)}</span>
            </span>
          )}
          {crew.paused && (
            <span className="flex items-center gap-1 font-medium text-quiet text-xs">
              <Pause aria-hidden size={12} />
              {t.crews.paused}
            </span>
          )}
        </button>
      </div>
      <ul>
        {shown.map((bot) => (
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
    case "question":
      return t.questions.activity(text);
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
