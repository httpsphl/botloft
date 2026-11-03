// The Botloft mascot in the bot's own color (spec 15.3). The color only
// tells bots apart; state is shown elsewhere with an icon and a label.
//
// The artwork (mascotArt.ts) is a flat body with soft shading layers clipped
// to its outline, re-tinted as a whole to the bot's color. A bot's avatar
// has no background: just the mascot, with a thin edge so light colors keep
// their shape on the light theme. `framed` puts the mascot on the black
// square of the app icon, for Botloft itself (the title bar, messages from
// the daemon), in the icon's own color.
//
// Given a `mood`, the mascot comes alive (mascot.css, flame.ts): its flame burns
// wildly, throwing embers, while it works; it sleeps as a small ember with
// its eyes shut while its bot has nothing to do, is paused or stopped; it
// hops as it starts to wait for the owner and burns low with heavy eyes when
// tired. Only the working flame moves on and on: everything else plays once.
// It makes short gestures (mascotMoments.ts, gestures.ts): it cheers when
// it finishes and falls asleep, wakes when it has something to do, pops in
// when its bot was just created, and glances at the bot it talks to
// (useGlance.ts), tossing it an ember (toss.ts). It plays with the mouse
// (pointer.ts): its eyes follow the cursor, it squeezes under it and hops
// when clicked. A bot's mascot also shows what its bot is doing, as a
// small badge by its head, and a face (mascotLook.ts, MascotBadge.tsx).
// `maker` dresses it as the Maker (spec 36.3, MakerGear.tsx).
// Awake, as on the pages that are not a bot's, it stands with its eyes open
// and blinks now and then. Without a mood it stays still, as in the chat
// history.

import { type CSSProperties, useEffect, useId, useMemo, useRef } from "react";
import type { Bot, BotId } from "../../lib/protocol.gen";
import { useFlame } from "./flame";
import { useGestures } from "./gestures";
import { MakerFlow, MakerHat, MakerWand } from "./MakerGear";
import { MascotBadge } from "./MascotBadge";
import { MascotEye, TILT } from "./MascotEye";
import { SCENE_BOX, WAND_BOX } from "./makerArt";
import {
  BODY,
  CENTER,
  EMBER,
  EYE_GLOW,
  EYES,
  OUTLINE,
  SHADES,
  SPARKS,
  VIEW_BOX,
} from "./mascotArt";
import type { Doing, Face } from "./mascotLook";
import { asleep, MOMENT_MS, useArrive, useMoment } from "./mascotMoments";
import { shiver, usePointerPlay } from "./pointer";
import { retint } from "./retint";
import { useGlance } from "./useGlance";

/**
 * `idle` is a bot with nothing to do: it sleeps, as a `sleeping` (paused
 * or stopped) one does. `awake` is a mascot that is no bot's.
 */
export type Mood = "awake" | "idle" | "working" | "waiting" | "tired" | "sleeping";

/** Botloft's own mascot, as on the app icon: the color that shows the drawing as drawn. */
export const BOTLOFT_COLOR = "#FF7A59";

/** How the mascot of `bot` moves, from what the bot is doing. */
export function moodOf(bot: Pick<Bot, "state" | "paused">, crewPaused = false): Mood {
  if (
    bot.paused ||
    crewPaused ||
    bot.state === "offline" ||
    bot.state === "sleeping" ||
    bot.state === "archived"
  ) {
    return "sleeping";
  }
  switch (bot.state) {
    case "busy":
    case "launching":
      return "working";
    case "needs_approval":
      return "waiting";
    case "rate_limited":
      return "tired";
    default:
      return "idle";
  }
}

/** How long the mascot cheers when its bot finishes (gestures.ts). */
export const CHEER_MS = MOMENT_MS.cheer;

/** The smallest mascot that shows its badge, and the smallest Maker with its wand. */
const BADGE_FROM = 24;
const WAND_FROM = 72;

/** Where the flame starts its turn, from the color, so a crew's flames do not burn together. */
function flameOffset(color: string): number {
  let hash = 0;
  for (const char of color) {
    hash = (hash * 31 + char.charCodeAt(0)) % 7000;
  }
  return hash;
}

/** The box the mascot takes: around the body, or wider for the Maker's wand or flow. */
function boxOf(framed: boolean, flow: boolean, wand: boolean): readonly number[] {
  if (framed) return [CENTER.x - 500, CENTER.y - 470, 1000, 1000];
  if (flow) return SCENE_BOX;
  return wand ? WAND_BOX : VIEW_BOX;
}

export function BotAvatar({
  color: chosen,
  size = 28,
  framed = false,
  mood,
  still = false,
  botId,
  arriving = false,
  starting = false,
  doing = null,
  face = null,
  maker = false,
  scene = false,
}: {
  color: string;
  size?: number;
  framed?: boolean;
  mood?: Mood | undefined;
  /** Shows the mood without moving. */
  still?: boolean;
  /** Whose mascot this is, so it can glance at the bots it talks to. */
  botId?: BotId | undefined;
  /** Pops in: its bot was just created. */
  arriving?: boolean;
  /** Its bot is starting up: done, it does not cheer as for finished work. */
  starting?: boolean;
  /** What its bot is doing, as a badge by its head (mascotLook.ts). */
  doing?: Doing | null;
  /** The shape of its eyes on top of the mood (mascotLook.ts). */
  face?: Face | null;
  /** Dressed as the Maker, in Botloft's own color (spec 36.3). */
  maker?: boolean;
  /** The Maker with its flow behind it, where there is room. */
  scene?: boolean;
}) {
  const color = maker ? BOTLOFT_COLOR : chosen;
  // Every avatar on the page needs its own clip, blur and gradients.
  const unique = useId().replace(/[^\w-]/g, "");
  const id = (name: string) => `avatar-${name}-${unique}`;
  const paint = useMemo(
    () => ({
      body: retint(BODY, color),
      shades: SHADES.map((shade) => retint(shade.fill, color)),
      emberLight: retint(EMBER.light, color),
      emberBase: retint(EMBER.base, color),
      glow: retint(EYE_GLOW, color),
    }),
    [color],
  );
  const svg = useRef<SVGSVGElement>(null);
  const moment = useMoment(mood, starting);
  const arrive = useArrive(arriving);
  const glance = useGlance(mood && !still ? botId : undefined, svg);
  useFlame(svg, mood === "working" && !still, flameOffset(color));
  useGestures(svg, mood && !still ? moment : null, Boolean(mood) && arrive, asleep(mood));
  usePointerPlay(svg, Boolean(mood) && !still);
  useEffect(() => {
    if (face === "worried" && !still && svg.current) shiver(svg.current);
  }, [face, still]);
  const style = glance
    ? ({ "--glance-x": `${glance.x}px`, "--glance-y": `${glance.y}px` } as CSSProperties)
    : undefined;
  const wand = maker && !framed && size >= WAND_FROM;
  const flow = maker && !framed && scene;
  const box = boxOf(framed, flow, wand);
  const scale = size / VIEW_BOX[3];
  return (
    <svg
      ref={svg}
      aria-hidden
      width={framed ? size : Math.round((box[2] ?? 0) * scale)}
      height={framed ? size : Math.round((box[3] ?? 0) * scale)}
      viewBox={box.join(" ")}
      className={`shrink-0 overflow-visible ${framed ? "rounded-lg bg-[#0b0b0b]" : ""}`}
      data-mood={mood}
      data-still={still || undefined}
      data-bot={botId}
      data-color={color}
      data-face={face ?? undefined}
      data-maker={maker || undefined}
      data-cheer={moment === "cheer" || undefined}
      data-wake={(moment === "wake" && !arrive) || undefined}
      data-sleep={moment === "sleep" || undefined}
      data-arrive={(mood && arrive) || undefined}
      data-glance={glance ? "" : undefined}
      style={style}
    >
      <defs>
        <clipPath id={id("clip")}>
          <path className="avatar-fire" d={OUTLINE} />
        </clipPath>
        <filter id={id("soft")} x="-10%" y="-10%" width="120%" height="120%">
          <feGaussianBlur stdDeviation="1.8" />
        </filter>
        <linearGradient id={id("ember")} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={paint.emberLight} />
          <stop offset="1" stopColor={paint.emberBase} />
        </linearGradient>
        <radialGradient id={id("eye")} cx="0.42" cy="1" r="0.72">
          <stop offset="0" stopColor={paint.glow} />
          <stop offset="0.45" stopColor="#2a0805" />
          <stop offset="1" stopColor="#0a0101" />
        </radialGradient>
      </defs>
      {flow && <MakerFlow arrowId={id("arrow")} ember={paint.body} />}
      <g className="avatar-hop">
        {/* The moments move the body in the spring, the mouse in the poke (gestures.ts, pointer.ts). */}
        <g className="avatar-spring">
          <g className="avatar-poke">
            {!framed && (
              <path
                className="avatar-fire"
                d={OUTLINE}
                fill="none"
                stroke="var(--avatar-edge)"
                strokeWidth={2}
                vectorEffect="non-scaling-stroke"
              />
            )}
            <g clipPath={`url(#${id("clip")})`}>
              <rect x={-200} y={0} width={1000} height={900} fill={paint.body} />
              <g filter={`url(#${id("soft")})`}>
                {SHADES.map((shade, index) => (
                  <path
                    key={shade.fill}
                    d={shade.d}
                    fill={paint.shades[index]}
                    fillRule="evenodd"
                  />
                ))}
              </g>
            </g>
            {mood === "working" &&
              SPARKS.map((spark) => (
                <g key={spark.x} transform={`translate(${spark.x} ${spark.y}) scale(1.5)`}>
                  {/* flame.ts moves it, as its spark says. */}
                  <path className="avatar-spark" d={EMBER.d} fill={`url(#${id("ember")})`} />
                </g>
              ))}
            <g className="avatar-look">
              <g className="avatar-gaze">
                {EYES.map((eye, index) => (
                  <MascotEye
                    key={eye.x}
                    eye={eye}
                    side={index === 0 ? "l" : "r"}
                    fill={`url(#${id("eye")})`}
                  />
                ))}
              </g>
            </g>
            {maker && <MakerHat tilt={TILT} />}
            {wand && <MakerWand ember={paint.body} />}
            {doing && size >= BADGE_FROM && <MascotBadge doing={doing} />}
          </g>
        </g>
      </g>
    </svg>
  );
}
