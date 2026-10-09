// The daily summary (spec 29.2): a routine for the crew's chief, which asks it
// each evening to tell the owner what the bots did. It is a routine like any
// other, shown below in the list; this card is the easy way to make it and to
// turn it on or off. The app knows it by the tool its request names.

import { Newspaper } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { routinesOf } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { describeSchedule, systemZone } from "./describe";
import { RoutineDialog } from "./RoutineDialog";

const TOOL = "crew_activity";
const EVERY_DAY = [1, 2, 3, 4, 5, 6, 7];

export function DailySummary({ crew }: { crew: Crew }) {
  const t = useT();
  const s = t.routines.summary;
  const api = useApi();
  const putRoutine = useApp((state) => state.putRoutine);
  const chief = useApp((state) => (crew.leadBotId ? state.bots[crew.leadBotId] : undefined));
  const mine = useApp(useShallow((state) => (chief ? routinesOf(state, chief.id) : [])));
  const existing = mine.find((routine) => routine.prompt.includes(TOOL));
  const [time, setTime] = useState("18:00");
  const [busy, setBusy] = useState(false);
  const [editing, setEditing] = useState(false);
  if (!chief) {
    return null;
  }

  const run = async (action: () => Promise<unknown>) => {
    setBusy(true);
    await attempt(s.failed, action);
    setBusy(false);
  };
  const create = () =>
    run(async () =>
      putRoutine(
        await api.call("routines.create", {
          botId: chief.id,
          name: s.name,
          prompt: s.prompt,
          schedule: { kind: "weekly", days: EVERY_DAY, time },
          timezone: systemZone(),
          overlap: "skip",
          missed: "run_once",
        }),
      ),
    );
  const toggle = (enabled: boolean) =>
    run(async () => {
      if (existing) {
        putRoutine(await api.call("routines.setEnabled", { routineId: existing.id, enabled }));
      }
    });

  return (
    <section
      aria-label={s.title}
      className="mb-3 flex max-w-3xl flex-col gap-2 rounded-xl border border-line bg-panel p-4"
    >
      <div className="flex items-start gap-3">
        <Newspaper aria-hidden size={20} className="mt-0.5 shrink-0 text-muted" />
        <div className="min-w-0 flex-1">
          <h3 className="font-semibold">{s.title}</h3>
          <p className="text-ink-soft text-sm">{s.hint}</p>
        </div>
      </div>
      {existing ? (
        <div className="flex flex-wrap items-center gap-2">
          <span className={`text-sm ${existing.enabled ? "text-ok" : "text-muted"}`}>
            {existing.enabled ? s.on(describeSchedule(existing.schedule, t.routines.when)) : s.off}
          </span>
          <Button size="sm" disabled={busy} onClick={() => toggle(!existing.enabled)}>
            {existing.enabled ? s.turnOff : s.turnOn}
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setEditing(true)}>
            {s.change}
          </Button>
        </div>
      ) : (
        <div className="flex flex-wrap items-center gap-2">
          <label className="flex items-center gap-2 text-sm">
            <span className="text-ink-soft">{s.time}</span>
            <input
              type="time"
              value={time}
              onChange={(event) => setTime(event.target.value)}
              className="h-8 rounded-lg border border-line-strong bg-canvas px-2 text-ink outline-none focus:border-accent"
            />
          </label>
          <Button size="sm" variant="primary" disabled={busy || !time} onClick={create}>
            {s.turnOn}
          </Button>
        </div>
      )}
      {editing && existing && (
        <RoutineDialog bot={chief} routine={existing} onClose={() => setEditing(false)} />
      )}
    </section>
  );
}
