// A page to record the mascots from (`?reel`, dev only): a row of bots in
// many colors passing by, as in a carousel. Every beat the row slides one
// place; the bot that comes to the middle lands, works for a moment and
// cheers when it finishes, with the same moods as in the app (BotAvatar).
// The others idle, blinking and glancing each in its own time.
// `scripts/reel.mjs` records it to a video.

import { type CSSProperties, useEffect, useState } from "react";
import { BotAvatar, CHEER_MS, type Mood } from "../features/bots/BotAvatar";
import "./reel.css";

const COLORS = [
  "#ff7a59",
  "#5ec8ff",
  "#8b6bff",
  "#4ade5c",
  "#ff5fa8",
  "#3fe0c5",
  "#ffcf3f",
  "#ff4d4d",
];

const params = new URLSearchParams(window.location.search);
/** Milliseconds between two slides (`?reel&beat=3000`). */
const BEAT = Number(params.get("beat")) || 3000;
/** When the bot in the middle starts working, once it has landed. */
const WORK_FROM = 450;
/** When it finishes, leaving its cheer time before the next slide. */
const WORK_UNTIL = Math.max(WORK_FROM + 400, BEAT - CHEER_MS - 250);
/** How many places show on each side of the middle. */
const REACH = 3;

type Phase = "arrive" | "work" | "done";

/** Where a place sits, in gaps from the middle: the neighbors stand closer together. */
function across(offset: number): number {
  if (offset === 0) return 0;
  return Math.sign(offset) * (1 + (Math.abs(offset) - 1) * 0.72);
}

export function Reel() {
  const [step, setStep] = useState(0);
  const [phase, setPhase] = useState<Phase>("arrive");
  const [size] = useState(() => Math.round(window.innerHeight * 0.36));

  useEffect(() => {
    document.documentElement.dataset.theme = "dark";
  }, []);

  useEffect(() => {
    setPhase("arrive");
    const timers = [
      setTimeout(() => setPhase("work"), WORK_FROM),
      setTimeout(() => setPhase("done"), WORK_UNTIL),
      setTimeout(() => setStep(step + 1), BEAT),
    ];
    return () => {
      for (const timer of timers) clearTimeout(timer);
    };
  }, [step]);

  const places: number[] = [];
  for (let place = step - REACH - 1; place <= step + REACH + 1; place++) {
    places.push(place);
  }
  return (
    <main className="reel" style={{ "--hero": `${size}px` } as CSSProperties}>
      {places.map((place) => {
        const offset = place - step;
        const center = offset === 0;
        const mood: Mood = center && phase === "work" ? "working" : "idle";
        const color = COLORS[((place % COLORS.length) + COLORS.length) % COLORS.length];
        return (
          <div
            key={place}
            className="reel-place"
            data-center={center || undefined}
            data-out={Math.abs(offset) > REACH || undefined}
            style={{ "--across": across(offset) } as CSSProperties}
          >
            <div className="reel-land">
              <BotAvatar color={color ?? COLORS[0] ?? ""} size={size} mood={mood} />
            </div>
          </div>
        );
      })}
    </main>
  );
}
