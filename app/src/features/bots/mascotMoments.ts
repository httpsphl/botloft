// The mascot's moments (spec 15.3): short gestures it makes once, on top of
// its mood, when something happens to its bot. It cheers when the bot
// finishes what it was doing (mascot-cheer.css), wakes when it comes back
// from paused or stopped, and pops in when the bot was just created
// (mascot-wake.css). Each lasts as long as its animation, then the mood
// takes over again.

import { useEffect, useState } from "react";
import type { Mood } from "./BotAvatar";

export type Moment = "cheer" | "wake";

/** How long each moment lasts, as in its CSS. */
export const MOMENT_MS: Record<Moment, number> = { cheer: 1200, wake: 800 };

/** How long a new bot's mascot pops in, as in mascot-wake.css. */
export const ARRIVE_MS = 700;

interface Seen {
  mood: Mood | undefined;
  /** The bot was starting up (launching), which also shows as working. */
  starting: boolean;
}

/**
 * The moment a change of mood calls for, if any. Only a bot that finished
 * real work cheers: one that just finished starting up does not.
 */
function momentOf(last: Seen, mood: Mood | undefined): Moment | null {
  if (last.mood === "working" && !last.starting && mood === "idle") return "cheer";
  if (last.mood === "sleeping" && mood !== undefined && mood !== "sleeping") return "wake";
  return null;
}

/**
 * The moment the mascot is in now: from the change of mood that calls for
 * it, for as long as it lasts. One that shows up in a mood makes none.
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
    const timer = setTimeout(() => setMoment(null), MOMENT_MS[moment]);
    return () => clearTimeout(timer);
  }, [moment]);
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
