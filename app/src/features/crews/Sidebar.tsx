import { Pause, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Activity, Bot, Crew } from "../../lib/protocol.gen";
import { botsOf, crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { AccountArea } from "../account/AccountArea";
import { BotAvatar } from "../bots/BotAvatar";
import { BotStateBadge, stateView } from "../bots/BotStateBadge";
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
        <ul className="min-h-0 flex-1 overflow-y-auto pb-3">
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

const row = "relative flex w-full items-center text-left hover:bg-sunken";
const marker = "before:absolute before:inset-y-0 before:left-0 before:w-[3px] before:bg-accent";

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
        className={`${row} h-8 gap-2 pr-3 pl-4 font-semibold text-sm ${selected ? `bg-sunken ${marker}` : ""}`}
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

function Conversation({ bot, crew }: { bot: Bot; crew: Crew }) {
  const t = useT();
  const selected = useApp((state) => state.selectedBotId === bot.id);
  const selectBot = useApp((state) => state.selectBot);
  const activity = bot.lastActivity;
  return (
    <li>
      <button
        type="button"
        aria-current={selected ? "page" : undefined}
        aria-label={`${bot.name}, ${stateView(bot, crew.paused).label}`}
        onClick={() => selectBot(bot.id)}
        className={`${row} gap-2.5 py-2 pr-3 pl-4 ${selected ? `bg-sunken ${marker}` : ""}`}
      >
        <BotAvatar color={bot.color} size={32} />
        <span className="min-w-0 flex-1">
          <span className="flex items-baseline gap-2">
            <span className="min-w-0 flex-1 truncate font-medium text-sm">{bot.name}</span>
            {activity && (
              <time
                className="shrink-0 text-muted text-xs"
                dateTime={new Date(activity.at).toISOString()}
              >
                {when(activity.at)}
              </time>
            )}
          </span>
          <span className="mt-0.5 flex min-w-0 items-center gap-1.5 text-xs">
            <BotStateBadge bot={bot} crewPaused={crew.paused} compact />
            <span className="min-w-0 truncate text-muted">
              {activity
                ? activityText(activity, t.crews.sidebar)
                : bot.role || t.crews.sidebar.noMessages}
            </span>
          </span>
        </span>
      </button>
    </li>
  );
}

/** The conversation-list line, worded here from what the daemon sends. */
function activityText(activity: Activity, t: Messages["crews"]["sidebar"]): string {
  switch (activity.kind) {
    case "owner":
      return t.fromOwner(activity.text);
    case "approval":
      return t.awaitingApproval(activity.text);
    default:
      return activity.text;
  }
}
