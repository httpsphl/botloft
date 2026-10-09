// The routines of a bot, or of a whole crew (spec 20.9), with a way to
// create one for a bot.

import { AlarmClock, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { botsOf, routinesOf } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { DailySummary } from "./DailySummary";
import { RoutineDialog } from "./RoutineDialog";
import { RoutineRow } from "./RoutineRow";

/** A bot's routines, as a tab of its page. */
export function BotRoutines({ bot }: { bot: Bot }) {
  const r = useT().routines;
  const routines = useApp(useShallow((state) => routinesOf(state, bot.id)));
  const [creating, setCreating] = useState(false);
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto flex max-w-3xl flex-col gap-3 px-5 py-5">
        {routines.length === 0 ? (
          <div className="flex flex-col items-center gap-2 rounded-2xl border border-line border-dashed px-6 py-10 text-center">
            <AlarmClock aria-hidden size={24} className="text-muted" />
            <p className="font-medium">{r.empty(bot.name)}</p>
            <p className="max-w-md text-muted text-sm">{r.emptyHint(bot.name)}</p>
            <Button
              className="mt-2"
              variant="primary"
              icon={Plus}
              onClick={() => setCreating(true)}
            >
              {r.newRoutine}
            </Button>
          </div>
        ) : (
          <>
            <div className="flex justify-end">
              <Button variant="primary" icon={Plus} onClick={() => setCreating(true)}>
                {r.newRoutine}
              </Button>
            </div>
            <ul aria-label={r.tab} className="flex flex-col gap-2">
              {routines.map((routine) => (
                <RoutineRow key={routine.id} routine={routine} bot={bot} showBot={false} />
              ))}
            </ul>
          </>
        )}
      </div>
      {creating && <RoutineDialog bot={bot} onClose={() => setCreating(false)} />}
    </div>
  );
}

/** Every routine of the crew's bots, as a tab of the crew's page. */
export function CrewRoutines({ crew }: { crew: Crew }) {
  const r = useT().routines;
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  // Routines themselves, so the selection stays equal between renders.
  const routines = useApp(useShallow((state) => bots.flatMap((bot) => routinesOf(state, bot.id))));
  const byBot = routines.flatMap((routine) => {
    const bot = bots.find((candidate) => candidate.id === routine.botId);
    return bot ? [{ bot, routine }] : [];
  });
  return (
    <div className="min-h-0 flex-1 overflow-y-auto p-5">
      <DailySummary crew={crew} />
      {byBot.length === 0 ? (
        <p className="rounded-2xl border border-line border-dashed px-6 py-10 text-center text-muted text-sm">
          {r.crewEmpty}
        </p>
      ) : (
        <ul aria-label={r.tab} className="flex max-w-3xl flex-col gap-2">
          {byBot.map(({ bot, routine }) => (
            <RoutineRow key={routine.id} routine={routine} bot={bot} showBot />
          ))}
        </ul>
      )}
    </div>
  );
}
