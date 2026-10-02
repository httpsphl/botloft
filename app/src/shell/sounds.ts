// When the app plays a sound (spec 15.1). The chimes of the notifications
// follow "Play a sound"; the others ("App sounds") mark a message sent, a
// reply or a file in the open chat, and a new bot or crew. Only while
// Botloft is in front, and never two within a moment, so many bots at once
// do not make a choir.

import { useEffect } from "react";
import type { BotId, CrewId } from "../lib/protocol.gen";
import { useAppStore, useHost } from "../store/context";
import { prefs } from "./prefs";
import { type Sound, tones } from "./tones";

/** Nothing plays this soon after another sound, except a bot that needs the owner. */
const QUIET_MS = 1500;

let lastAt = Number.NEGATIVE_INFINITY;

/** Whether `sound` may play at `now`; plays nothing. */
export function mayPlay(sound: Sound, now: number): boolean {
  const alert = sound === "needs" || sound === "done";
  if (!(alert ? prefs.sound.get() : prefs.appSounds.get())) {
    return false;
  }
  return sound === "needs" || now - lastAt >= QUIET_MS;
}

export function playSound(sound: Sound): void {
  const now = Date.now();
  if (!mayPlay(sound, now)) {
    return;
  }
  lastAt = now;
  tones(sound);
}

/** For tests: the next sound plays whenever. */
export function resetSounds(): void {
  lastAt = Number.NEGATIVE_INFINITY;
}

/** A spark for each new bot or crew, and a pop for a reply in the open chat. */
export function useAppSounds(): void {
  const host = useHost();
  const store = useAppStore();

  useEffect(() => {
    let bots = new Set<BotId>();
    let crews = new Set<CrewId>();
    let replies: Record<BotId, number> = {};
    let ready = false;
    const take = () => {
      const state = store.getState();
      bots = new Set(Object.keys(state.bots) as BotId[]);
      crews = new Set(Object.keys(state.crews) as CrewId[]);
      replies = state.replyAt;
      const wasReady = ready;
      ready = state.loaded;
      return wasReady;
    };
    take();
    return store.subscribe((state) => {
      const before = { bots, crews, replies };
      // What was there on (re)connecting is not news.
      if (!take() || !state.loaded || !host.window.inFront()) {
        return;
      }
      const born =
        [...bots].some((id) => !before.bots.has(id)) ||
        [...crews].some((id) => !before.crews.has(id));
      const open = state.selectedBotId;
      const replied = open !== null && (replies[open] ?? 0) > (before.replies[open] ?? 0);
      if (born) {
        playSound("spark");
      } else if (replied) {
        playSound("reply");
      }
    });
  }, [host, store]);
}
