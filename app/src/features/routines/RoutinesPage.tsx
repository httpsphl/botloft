// Every routine in one place (spec 20.9): what runs next across all crews,
// in the order it will run, and below, each bot's routines under its
// mascot, folded by the owner if they like.

import { AlarmClock, ChevronDown, LoaderCircle, Plus, Radar } from "lucide-react";
import { useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { AVATAR_PALETTE, type Bot, type Routine } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { EmptyState } from "../../ui/EmptyState";
import { BotAvatar } from "../bots/BotAvatar";
import { describeSchedule, nextText } from "./describe";
import { RoutineDialog } from "./RoutineDialog";
import { RoutineRow } from "./RoutineRow";

/** How many upcoming routines show before "Show more". */
const FIRST = 5;

interface Owned {
  bot: Bot;
  routine: Routine;
}

const capitalize = (text: string) => text.charAt(0).toLocaleUpperCase() + text.slice(1);

const byCreation = (a: { createdAt: number; id: string }, b: { createdAt: number; id: string }) =>
  a.createdAt - b.createdAt || a.id.localeCompare(b.id);

/** The bots with routines, crew by crew, each with its routines. */
function useGroups(): { bot: Bot; crew: string; routines: Routine[] }[] {
  // The store's own objects, so the selections stay equal between renders.
  const crews = useApp(useShallow(crewList));
  const bots = useApp(useShallow((state) => Object.values(state.bots)));
  const routines = useApp(useShallow((state) => Object.values(state.routines)));
  return useMemo(
    () =>
      crews.flatMap((crew) =>
        bots
          .filter((bot) => bot.crewId === crew.id)
          .sort(byCreation)
          .flatMap((bot) => {
            const own = routines.filter((routine) => routine.botId === bot.id).sort(byCreation);
            return own.length ? [{ bot, crew: crew.name, routines: own }] : [];
          }),
      ),
    [crews, bots, routines],
  );
}

/** Routines that run on time, soonest first; signals wait, so they are left out. */
function upcoming(groups: { bot: Bot; routines: Routine[] }[]): Owned[] {
  return groups
    .flatMap(({ bot, routines }) => routines.map((routine) => ({ bot, routine })))
    .filter(
      ({ routine }) =>
        routine.enabled && routine.nextRunAt !== null && routine.schedule.kind !== "signal",
    )
    .sort((a, b) => (a.routine.nextRunAt ?? 0) - (b.routine.nextRunAt ?? 0));
}

function Upcoming({ bot, routine }: Owned) {
  const r = useT().routines;
  const [editing, setEditing] = useState(false);
  const schedule = describeSchedule(routine.schedule, r.when);
  // A routine every few minutes is keeping watch more than it is due.
  const watching = routine.schedule.kind === "interval";
  const running = routine.lastRun?.status === "queued";
  const when = watching
    ? r.page.watching
    : capitalize(nextText(routine.nextRunAt ?? 0, routine.timezone, r));
  return (
    <li>
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left transition-colors hover:bg-sunken"
      >
        <BotAvatar color={bot.color} size={28} mood={running ? "working" : undefined} />
        <span className="min-w-0 flex-1">
          <span className="block truncate font-medium">{routine.name}</span>
          <span className="flex items-center gap-1.5 truncate text-muted text-sm">
            {running ? (
              <>
                <LoaderCircle aria-hidden size={12} className="shrink-0 animate-spin text-work" />
                <span className="text-work">{r.last.queued}</span>
              </>
            ) : (
              <>
                {watching && <Radar aria-hidden size={12} className="shrink-0 text-ok" />}
                <span className={watching ? "text-ok" : "text-ink-soft"}>{when}</span>
              </>
            )}
            <span aria-hidden>·</span>
            <span className="truncate">{schedule}</span>
          </span>
        </span>
        <span className="shrink-0 text-muted text-xs">{bot.name}</span>
      </button>
      {editing && <RoutineDialog bot={bot} routine={routine} onClose={() => setEditing(false)} />}
    </li>
  );
}

function BotGroup({ bot, crew, routines }: { bot: Bot; crew: string; routines: Routine[] }) {
  const r = useT().routines;
  const [open, setOpen] = useState(true);
  const [creating, setCreating] = useState(false);
  return (
    <section aria-label={bot.name} className="flex flex-col gap-2">
      <div className="flex items-center gap-1">
        <button
          type="button"
          aria-expanded={open}
          onClick={() => setOpen(!open)}
          className="flex min-w-0 flex-1 items-center gap-2 rounded-lg px-2 py-1.5 text-left transition-colors hover:bg-sunken"
        >
          <BotAvatar color={bot.color} size={22} />
          <span className="truncate font-semibold">{bot.name}</span>
          <span className="truncate text-muted text-xs">{crew}</span>
          <ChevronDown
            aria-hidden
            size={14}
            className={`shrink-0 text-muted transition-transform ${open ? "" : "-rotate-90"}`}
          />
        </button>
        <Button
          variant="ghost"
          size="sm"
          icon={Plus}
          label={r.dialog.newTitle(bot.name)}
          onClick={() => setCreating(true)}
        />
      </div>
      {open && (
        <ul aria-label={r.page.of(bot.name)} className="flex flex-col gap-2">
          {routines.map((routine) => (
            <RoutineRow key={routine.id} routine={routine} bot={bot} showBot={false} />
          ))}
        </ul>
      )}
      {creating && <RoutineDialog bot={bot} onClose={() => setCreating(false)} />}
    </section>
  );
}

export function RoutinesPage() {
  const r = useT().routines;
  const groups = useGroups();
  const next = upcoming(groups);
  const [all, setAll] = useState(false);
  const shown = all ? next : next.slice(0, FIRST);
  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="mx-auto flex max-w-3xl flex-col gap-8 px-5 py-6">
        <header>
          <h1 className="font-semibold text-xl">{r.page.title}</h1>
          <p className="text-muted text-sm">{r.page.lead}</p>
        </header>
        {groups.length === 0 ? (
          <EmptyState
            color={AVATAR_PALETTE[0]}
            icon={AlarmClock}
            title={r.page.emptyTitle}
            body={r.page.emptyBody}
          />
        ) : (
          <>
            {next.length > 0 && (
              <section aria-labelledby="routines-upcoming" className="flex flex-col gap-1">
                <h2
                  id="routines-upcoming"
                  className="px-3 font-medium text-muted text-xs uppercase tracking-wide"
                >
                  {r.page.upcoming}
                </h2>
                <ul aria-labelledby="routines-upcoming" className="flex flex-col">
                  {shown.map((item) => (
                    <Upcoming key={item.routine.id} {...item} />
                  ))}
                </ul>
                {next.length > FIRST && (
                  <button
                    type="button"
                    onClick={() => setAll(!all)}
                    className="self-start rounded-lg px-3 py-1.5 text-muted text-sm transition-colors hover:bg-sunken hover:text-ink"
                  >
                    {all ? r.page.showLess : r.page.showMore(next.length - FIRST)}
                  </button>
                )}
              </section>
            )}
            <div className="flex flex-col gap-6">
              {groups.map((group) => (
                <BotGroup key={group.bot.id} {...group} />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
