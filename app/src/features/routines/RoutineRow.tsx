// One routine in a list (spec 20.9): on or off, its name, when it runs,
// the next time and how the last run went, with Run now and a menu.

import {
  CircleAlert,
  CircleCheck,
  Ellipsis,
  LoaderCircle,
  Pencil,
  Play,
  SkipForward,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { type Messages, useT } from "../../i18n";
import type { Bot, Routine, RoutineRun } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Menu } from "../../ui/Menu";
import { Switch } from "../../ui/Switch";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { describeSchedule, nextText } from "./describe";
import { RoutineDialog } from "./RoutineDialog";

export function RoutineRow({
  routine,
  bot,
  showBot,
}: {
  routine: Routine;
  bot: Bot;
  showBot: boolean;
}) {
  const r = useT().routines;
  const api = useApi();
  const putRoutine = useApp((state) => state.putRoutine);
  const [open, setOpen] = useState<"edit" | "remove" | null>(null);
  const close = () => setOpen(null);

  const toggle = () =>
    attempt(r.failed.toggle, async () =>
      putRoutine(
        await api.call("routines.setEnabled", {
          routineId: routine.id,
          enabled: !routine.enabled,
        }),
      ),
    );
  const runNow = () =>
    attempt(r.failed.runNow, () => api.call("routines.runNow", { routineId: routine.id }));

  return (
    <li className="flex items-start gap-3 rounded-xl border border-line bg-panel px-4 py-3">
      <Switch
        checked={routine.enabled}
        label={r.toggle(routine.name)}
        onChange={toggle}
        className="mt-0.5"
      />
      <div className="min-w-0 flex-1">
        {showBot && (
          <p className="mb-0.5 flex items-center gap-1.5 text-muted text-xs">
            <BotAvatar color={bot.color} size={14} />
            {bot.name}
          </p>
        )}
        <p className={`truncate font-medium ${routine.enabled ? "text-ink" : "text-muted"}`}>
          {routine.name}
        </p>
        <p className="text-ink-soft text-sm">
          {describeSchedule(routine.schedule, r.when)}
          <span className="text-muted">
            {" · "}
            {routine.enabled && routine.nextRunAt !== null
              ? r.next(nextText(routine.nextRunAt, routine.timezone, r))
              : r.off}
          </span>
        </p>
        {routine.lastRun && <LastRun run={routine.lastRun} words={r.last} />}
      </div>
      <Button size="sm" icon={Play} onClick={runNow}>
        {r.runNow}
      </Button>
      <Menu
        label={r.more(routine.name)}
        icon={Ellipsis}
        items={[
          { label: r.edit, icon: Pencil, onSelect: () => setOpen("edit") },
          { label: r.remove, icon: Trash2, danger: true, onSelect: () => setOpen("remove") },
        ]}
      />
      {open === "edit" && <RoutineDialog bot={bot} routine={routine} onClose={close} />}
      {open === "remove" && (
        <Confirm
          title={r.removeTitle(routine.name)}
          confirmLabel={r.remove}
          onClose={close}
          onConfirm={() =>
            attempt(r.failed.remove, async () =>
              putRoutine(await api.call("routines.archive", { routineId: routine.id })),
            )
          }
        >
          {r.removeBody}
        </Confirm>
      )}
    </li>
  );
}

function LastRun({ run, words }: { run: RoutineRun; words: Messages["routines"]["last"] }) {
  const line = "mt-1 flex items-center gap-1.5 text-xs";
  switch (run.status) {
    case "queued":
      return (
        <p className={`${line} text-work`}>
          <LoaderCircle aria-hidden size={12} className="animate-spin" />
          {words.queued}
        </p>
      );
    case "done":
      return (
        <p className={`${line} text-ok`}>
          <CircleCheck aria-hidden size={12} />
          {words.done}
        </p>
      );
    case "failed":
      return (
        <p className={`${line} text-danger`}>
          <CircleAlert aria-hidden size={12} />
          {words.failed}
        </p>
      );
    case "skipped": {
      const text =
        run.reason === "missed"
          ? words.missed(run.skippedCount)
          : run.reason === "bot_paused"
            ? words.bot_paused
            : words.overlap;
      return (
        <p className={`${line} text-muted`}>
          <SkipForward aria-hidden size={12} />
          {text}
        </p>
      );
    }
  }
}
