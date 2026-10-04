// The mascot's moments (spec 15.3): short gestures it makes once, on top of
// its mood, when something happens to its bot. Only a working bot's flame
// moves on and on; every change between the other moods is one of these.
// It cheers when the bot finishes what it was doing (mascot-cheer.css),
// wakes when it has something to do or comes back from paused or stopped,
// pops in when the bot was just created (mascot-wake.css), and falls asleep
// when it has nothing left to do (mascot-sleep.css). Each lasts as long as
// its animation, then the mood takes over again.

import { useEffect, useState } from "react";
import type { Mood } from "./BotAvatar";

export type Moment = "cheer" | "wake" | "sleep";

/** How long each moment lasts, as in its CSS. */
export const MOMENT_MS: Record<Moment, number> = { cheer: 1200, wake: 800, sleep: 900 };

/** How long a new bot's mascot pops in, as in mascot-wake.css. */
export const ARRIVE_MS = 700;

interface Seen {
  mood: Mood | undefined;
  /** The bot was starting up (launching), which also shows as working. */
  starting: boolean;
}

/** A bot with nothing to do sleeps as a paused or stopped one does. */
export function asleep(mood: Mood | undefined): boolean {
  return mood === "idle" || mood === "sleeping";
}

/**
 * The moment a change of mood calls for, if any. Only a bot that finished
 * real work cheers: one that just finished starting up does not. One
 * turned back on wakes, even when it has nothing to do and goes back to
 * sleep right after.
 */
function momentOf(last: Seen, mood: Mood | undefined): Moment | null {
  if (last.mood === undefined || mood === undefined) return null;
  if (last.mood === "working" && !last.starting && mood === "idle") return "cheer";
  if (last.mood === "sleeping" && mood !== "sleeping") return "wake";
  if (asleep(last.mood)) return asleep(mood) ? null : "wake";
  return asleep(mood) ? "sleep" : null;
}

/**
 * The moment the mascot is in now: from the change of mood that calls for
 * it, for as long as it lasts. One that shows up in a mood makes none. A
 * mascot left asleep by its moment (a cheer, a wake) then falls asleep.
 */
export function useMoment(mood: Mood | undefined, starting = false): Moment | null {
  const [last, setLast] = useState<Seen>({ mood, starting });
  const [moment, setMoment] = useState<Moment | null>(null);
  if (mood !== last.mood || starting !== last.starting) {
    setLast({ mood, starting });
    if (mood !== last.mood) setMoment(momentOf(last, mood));
  }
  useEffect(() => {
    if (!moment) return;
    const next = moment !== "sleep" && asleep(mood) ? "sleep" : null;
    const timer = setTimeout(() => setMoment(next), MOMENT_MS[moment]);
    return () => clearTimeout(timer);
  }, [moment, mood]);
  return moment;
}

/** Whether the mascot is still popping in, when it showed up `arriving`. */
export function useArrive(arriving: boolean): boolean {
  const [now, setNow] = useState(arriving);
  useEffect(() => {
    if (!now) return;
    const timer = setTimeout(() => setNow(false), ARRIVE_MS);
    return () => clearTimeout(timer);
  }, [now]);
  return now;
}
