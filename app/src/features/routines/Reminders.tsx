// An agent's own returns to its work (spec 20.12), over its message field:
// "Scout comes back to it today at 15:30", with Cancel, which archives it.

import { AlarmClock, X } from "lucide-react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";
import { nextText } from "./describe";
import { useReminders } from "./useShortcuts";

export function Reminders({ bot }: { bot: Bot }) {
  const t = useT().routines;
  const api = useApi();
  const reminders = useReminders(bot.id);
  if (reminders.length === 0) {
    return null;
  }
  return (
    <ul aria-label={t.reminder.list(bot.name)} className="flex flex-col items-center gap-1.5">
      {reminders.map((routine) => (
        <li
          key={routine.id}
          className="flex items-center gap-2 whitespace-nowrap rounded-full border border-line bg-panel py-1 pr-1 pl-3 text-sm shadow-sm"
        >
          <AlarmClock aria-hidden size={14} className="shrink-0 text-muted" />
          <span title={routine.prompt}>
            {t.reminder.label(bot.name, nextText(routine.nextRunAt ?? 0, routine.timezone, t))}
          </span>
          <button
            type="button"
            aria-label={t.reminder.cancel(routine.name)}
            title={t.reminder.cancel(routine.name)}
            onClick={() =>
              attempt(t.reminder.failed, () =>
                api.call("routines.archive", { routineId: routine.id }),
              )
            }
            className="grid h-6 w-6 place-items-center rounded-full text-muted hover:bg-sunken hover:text-ink"
          >
            <X aria-hidden size={13} />
          </button>
        </li>
      ))}
    </ul>
  );
}
