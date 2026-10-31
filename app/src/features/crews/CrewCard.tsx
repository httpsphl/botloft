// A crew as a card on the home (spec 15.1): its faces, what they do, its
// unread count; a click opens the crew.

import { Pause } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApp } from "../../store/context";
import { unreadIn } from "../../store/seen";
import { CountBadge } from "../../ui/Badge";
import { BotAvatar, moodOf } from "../bots/BotAvatar";

/** Faces shown on a card; the rest are counted. */
const FACES = 6;

/** A crew as a card: its faces, what they do, its unread count; also on the home. */
export function CrewCard({ crew }: { crew: Crew }) {
  const t = useT();
  const words = t.crews.overview;
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const unread = useApp((state) => unreadIn(state, crew.id));
  const selectCrew = useApp((state) => state.selectCrew);
  const { working, waiting } = tally(bots);
  const parts = [
    waiting > 0 && words.waiting(waiting),
    working > 0 && words.working(working),
  ].filter((part): part is string => Boolean(part));
  const status = crew.paused
    ? t.crews.paused
    : parts.join(" · ") || (bots.length > 0 ? words.calm : words.noBots);
  return (
    <li>
      <button
        type="button"
        onClick={() => selectCrew(crew.id)}
        className="flex h-full w-full flex-col gap-3 rounded-xl border border-line bg-panel p-3.5 text-left transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-line-strong hover:shadow-sm"
      >
        <span className="flex w-full items-center gap-2">
          <span className="min-w-0 flex-1">
            <span className="block truncate font-semibold">{crew.name}</span>
            <span className="block text-muted text-xs">{t.crews.view.bots(bots.length)}</span>
          </span>
          {unread > 0 && (
            <CountBadge tone="quiet" count={unread} label={t.crews.sidebar.unreadCount(unread)} />
          )}
        </span>
        {bots.length > 0 && (
          <span aria-hidden className="flex items-center gap-1">
            {bots.slice(0, FACES).map((bot) => (
              // Still: a page of crews would otherwise repaint every flame.
              <BotAvatar
                key={bot.id}
                color={bot.color}
                size={26}
                mood={moodOf(bot, crew.paused)}
                still
              />
            ))}
            {bots.length > FACES && (
              <span className="ml-1 text-muted text-xs tabular-nums">+{bots.length - FACES}</span>
            )}
          </span>
        )}
        <span
          className={`flex items-center gap-1.5 text-xs ${waiting > 0 && !crew.paused ? "font-medium text-warn" : "text-muted"}`}
        >
          {crew.paused && <Pause aria-hidden size={12} />}
          {status}
        </span>
      </button>
    </li>
  );
}

function tally(bots: Bot[]): { working: number; waiting: number } {
  let working = 0;
  let waiting = 0;
  for (const bot of bots) {
    if (bot.state === "busy" || bot.state === "launching") {
      working += 1;
    } else if (bot.state === "needs_approval" || bot.state === "auth_error") {
      waiting += 1;
    }
  }
  return { working, waiting };
}
