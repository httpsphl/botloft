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

/** When a return comes back, or when a wait runs out. */
function soonest(routine: Routine): number {
  return routine.schedule.kind === "wait" ? routine.schedule.until : (routine.nextRunAt ?? 0);
}

/**
 * The agent's own returns to its work still to come (spec 20.12) and the
 * waits it set (20.14), soonest first.
 */
export function useReminders(bot: BotId): Routine[] {
  return useApp(
    useShallow((state) =>
      Object.values(state.routines)
        .filter(
          (routine) =>
            routine.botId === bot &&
            routine.archivedAt === null &&
            ((routine.schedule.kind === "once" &&
              routine.schedule.reminder === true &&
              routine.nextRunAt !== null) ||
              (routine.schedule.kind === "wait" && routine.enabled)),
        )
        .sort((a, b) => soonest(a) - soonest(b)),
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
