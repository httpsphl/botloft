// Routines that wait for the owner's click (spec 20.2): the shortcuts on
// the home and in an agent's chat, and an agent's own reminders to come
// back to its work, which the owner may cancel.

import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, BotId, Routine } from "../../lib/protocol.gen";
import type { AppState } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { attempt, notify } from "../../ui/toast";

/** The owner's shortcuts, by name; one agent's when `bot` is set. */
export function shortcutsOf(state: AppState, bot?: BotId): Routine[] {
  return Object.values(state.routines)
    .filter(
      (routine) =>
        routine.schedule.kind === "manual" &&
        routine.enabled &&
        routine.archivedAt === null &&
        (bot === undefined || routine.botId === bot),
    )
    .sort((a, b) => a.name.localeCompare(b.name));
}

/** The agent's own returns to its work still to come, soonest first. */
export function useReminders(bot: BotId): Routine[] {
  return useApp(
    useShallow((state) =>
      Object.values(state.routines)
        .filter(
          (routine) =>
            routine.botId === bot &&
            routine.schedule.kind === "once" &&
            routine.schedule.reminder === true &&
            routine.archivedAt === null &&
            routine.nextRunAt !== null,
        )
        .sort((a, b) => (a.nextRunAt ?? 0) - (b.nextRunAt ?? 0)),
    ),
  );
}

/** Runs a shortcut at once and says which agent took it. */
export function useRunShortcut() {
  const api = useApi();
  const words = useT().routines.shortcuts;
  return (routine: Routine, bot: Pick<Bot, "name"> | undefined) =>
    attempt(words.failed, async () => {
      await api.call("routines.runNow", { routineId: routine.id });
      notify(words.started(bot?.name ?? "", routine.name));
    });
}
