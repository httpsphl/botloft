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

/** The moment a change of mood calls for, if any. */
function momentOf(last: Mood | undefined, mood: Mood | undefined): Moment | null {
  if (last === "working" && mood === "idle") return "cheer";
  if (last === "sleeping" && mood !== undefined && mood !== "sleeping") return "wake";
  return null;
}

/**
 * The moment the mascot is in now: from the change of mood that calls for
 * it, for as long as it lasts. One that shows up in a mood makes none.
 */
export function useMoment(mood: Mood | undefined): Moment | null {
  const [last, setLast] = useState(mood);
  const [moment, setMoment] = useState<Moment | null>(null);
  if (mood !== last) {
    setLast(mood);
    setMoment(momentOf(last, mood));
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
