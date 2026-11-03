// On the island's Home (spec 31.4): the routines that wait for the owner's
// click (spec 20.2), from every crew, as on the app's home. A click runs
// one and it says it started.

import { Check, Play } from "lucide-react";
import { useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Bot, Routine } from "../lib/protocol.gen";

export function Shortcuts({
  api,
  shortcuts,
  bots,
}: {
  api: BotloftApi;
  shortcuts: Routine[];
  bots: Bot[];
}) {
  const t = useT().island.shortcuts;
  const [ran, setRan] = useState<string | null>(null);
  if (shortcuts.length === 0) {
    return null;
  }
  const run = (routine: Routine) =>
    api.call("routines.runNow", { routineId: routine.id }).then(
      () => setRan(routine.id),
      () => {},
    );
  return (
    <section className="island-card flex flex-col gap-2.5 p-4" aria-label={t.title}>
      <span className="flex items-center gap-1.5 font-semibold text-[15px]">
        <Play aria-hidden size={13} />
        {t.title}
      </span>
      <ul className="flex flex-wrap gap-1.5">
        {shortcuts.map((routine) => {
          const bot = bots.find((have) => have.id === routine.botId);
          const started = ran === routine.id;
          return (
            <li key={routine.id}>
              <button
                type="button"
                aria-label={t.run(routine.name, bot?.name ?? "")}
                disabled={started}
                onClick={() => void run(routine)}
                className="flex items-center gap-1.5 rounded-full bg-[var(--island-tab)] py-1 pr-3 pl-1 text-[13px] hover:brightness-125 disabled:opacity-70"
              >
                {bot && <BotAvatar color={bot.color} size={20} mood="idle" still />}
                {started ? (
                  <span className="flex items-center gap-1">
                    <Check aria-hidden size={13} className="text-[var(--island-ok)]" />
                    {t.started}
                  </span>
                ) : (
                  routine.name
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
