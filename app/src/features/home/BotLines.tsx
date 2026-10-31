// "Working now" and "New replies" on the home (spec 15.1): one line per
// agent of any crew, with what it does or did last, as its line in the
// conversation list says it; a click opens its chat.

import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { fromNow, when } from "../../lib/format";
import type { Bot } from "../../lib/protocol.gen";
import { activityOf } from "../../store/app";
import { useApp } from "../../store/context";
import { isUnread } from "../../store/seen";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import { activityText } from "../crews/SidebarConversation";

const working = (bot: Bot) => bot.state === "busy" || bot.state === "launching";

/** Busy agents of every crew. */
export function useWorking(): Bot[] {
  return useApp(useShallow((state) => Object.values(state.bots).filter(working)));
}

/** Agents with a reply the owner has not read, the newest reply first. */
export function useUnread(): Bot[] {
  return useApp(
    useShallow((state) =>
      Object.values(state.bots)
        .filter((bot) => isUnread(state, bot.id))
        .sort((a, b) => (state.replyAt[b.id] ?? 0) - (state.replyAt[a.id] ?? 0)),
    ),
  );
}

export function BotLines({ bots, time }: { bots: Bot[]; time: "since" | "reply" }) {
  return (
    <ul className="flex flex-col overflow-hidden rounded-xl border border-line bg-panel">
      {bots.map((bot) => (
        <Line key={bot.id} bot={bot} time={time} />
      ))}
    </ul>
  );
}

function Line({ bot, time }: { bot: Bot; time: "since" | "reply" }) {
  const t = useT();
  const crew = useApp((state) => state.crews[bot.crewId]);
  const activity = useApp((state) => activityOf(state, bot.id));
  const replyAt = useApp((state) => state.replyAt[bot.id]);
  const selectBot = useApp((state) => state.selectBot);
  const at = time === "reply" ? replyAt : activity?.at;
  return (
    <li className="border-line border-b last:border-b-0">
      <button
        type="button"
        onClick={() => selectBot(bot.id)}
        className="flex w-full items-center gap-2.5 px-3.5 py-2.5 text-left text-sm transition-colors hover:bg-sunken"
      >
        <BotAvatar color={bot.color} size={24} mood={moodOf(bot, crew?.paused)} still />
        <span className="shrink-0 font-semibold">{bot.name}</span>
        {crew && (
          <span className="shrink-0 text-muted text-xs">{t.questions.box.inCrew(crew.name)}</span>
        )}
        <span className="min-w-0 flex-1 truncate text-ink-soft">
          {activity ? activityText(activity, t) : bot.role}
        </span>
        {at !== undefined && (
          <time className="shrink-0 text-muted text-xs" dateTime={new Date(at).toISOString()}>
            {time === "reply" ? when(at) : fromNow(at)}
          </time>
        )}
      </button>
    </li>
  );
}
