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
// Given a `mood`, the mascot comes alive (mascot.css): its flame burns
// wildly, throwing embers, while it works; it sleeps as a small ember with
// its eyes shut while its bot has nothing to do, is paused or stopped; it
// hops as it starts to wait for the owner and burns low with heavy eyes when
// tired. Only the working flame moves on and on: everything else plays once.
// It makes short gestures (mascotMoments.ts): it cheers when it finishes
// and falls asleep, wakes when it has something to do, pops in when its bot
// was just created, and glances at the bot it talks to (useGlance.ts).
// Awake, as on the pages that are not a bot's, it stands with its eyes open
// and blinks now and then. Without a mood it stays still, as in the chat
// history.

import { type CSSProperties, useId, useMemo, useRef } from "react";
import type { Bot, BotId } from "../../lib/protocol.gen";
import {
  BODY,
  CENTER,
  type Ellipse,
  EMBER,
  EYE_GLOW,
  EYES,
  HIGHLIGHTS,
  OUTLINE,
  SHADES,
  SPARKS,
  VIEW_BOX,
} from "./mascotArt";
import { MOMENT_MS, useArrive, useMoment } from "./mascotMoments";
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
  if (bot.paused || crewPaused || bot.state === "offline" || bot.state === "archived") {
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

/** How long the mascot cheers when its bot finishes, as in mascot-cheer.css. */
export const CHEER_MS = MOMENT_MS.cheer;

/** A delay from the color, so a crew's mascots do not blink together. */
function blinkDelay(color: string): string {
  let hash = 0;
  for (const char of color) {
    hash = (hash * 31 + char.charCodeAt(0)) % 7000;
  }
  return `-${hash}ms`;
}

/** The glints that sit on `eye`. */
function glintsOn(eye: Ellipse): Ellipse[] {
  const reach = Math.max(eye.rx, eye.ry) * 1.3;
  return HIGHLIGHTS.filter((glint) => Math.hypot(glint.x - eye.x, glint.y - eye.y) < reach);
}

const rotate = ({ x, y, turn }: Ellipse) => `rotate(${turn} ${x} ${y})`;

/**
 * How far the face leans, in degrees: the line through the two eyes, the
 * right one sitting higher. Shut and smiling eyes lean with it.
 */
function tiltOf(eyes: readonly Ellipse[]): number {
  const [left, right] = eyes;
  if (!left || !right) return 0;
  return (Math.atan2(right.y - left.y, right.x - left.x) * 180) / Math.PI;
}

const TILT = tiltOf(EYES);

export function BotAvatar({
  color,
  size = 28,
  framed = false,
  mood,
  still = false,
  botId,
  arriving = false,
  starting = false,
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
}) {
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
  const style = mood
    ? ({
        "--blink-delay": blinkDelay(color),
        ...(glance && { "--glance-x": `${glance.x}px`, "--glance-y": `${glance.y}px` }),
      } as CSSProperties)
    : undefined;
  const box = framed ? [CENTER.x - 500, CENTER.y - 470, 1000, 1000] : VIEW_BOX;
  return (
    <svg
      ref={svg}
      aria-hidden
      width={size}
      height={size}
      viewBox={box.join(" ")}
      className={`shrink-0 overflow-visible ${framed ? "rounded-lg bg-[#0b0b0b]" : ""}`}
      data-mood={mood}
      data-still={still || undefined}
      data-bot={botId}
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
      <g className="avatar-hop">
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
              <path key={shade.fill} d={shade.d} fill={paint.shades[index]} fillRule="evenodd" />
            ))}
          </g>
        </g>
        {mood === "working" &&
          SPARKS.map((spark) => (
            <g key={spark.x} transform={`translate(${spark.x} ${spark.y}) scale(1.5)`}>
              <path
                className="avatar-spark"
                d={EMBER.d}
                fill={`url(#${id("ember")})`}
                style={
                  {
                    "--spark-delay": spark.delay,
                    "--spark-drift": spark.drift,
                    "--spark-turn": spark.turn,
                  } as CSSProperties
                }
              />
            </g>
          ))}
        <g className="avatar-look">
          {EYES.map((eye, index) => (
            <Eye key={eye.x} eye={eye} side={index === 0 ? "l" : "r"} fill={`url(#${id("eye")})`} />
          ))}
        </g>
      </g>
    </svg>
  );
}

/**
 * One eye: open with its glints, shut into a soft arc while asleep, or
 * bent up into a smile while the mascot cheers.
 */
function Eye({ eye, side, fill }: { eye: Ellipse; side: "l" | "r"; fill: string }) {
  const { x, y, rx, ry } = eye;
  return (
    <g>
      <g className={`avatar-eye avatar-eye-${side}`}>
        <ellipse cx={x} cy={y} rx={rx} ry={ry} transform={rotate(eye)} fill={fill} />
        <g className="avatar-hl">
          {glintsOn(eye).map((glint) => (
            <ellipse
              key={glint.x}
              cx={glint.x}
              cy={glint.y}
              rx={glint.rx}
              ry={glint.ry}
              transform={rotate(glint)}
              fill="#ffffff"
            />
          ))}
        </g>
      </g>
      {/* The arcs lean in a group of their own: the smile's CSS transform
          would replace a transform set on the path itself. */}
      <g transform={`rotate(${TILT} ${x} ${y})`}>
        <path
          className="avatar-closed"
          d={`M${x - rx * 0.8} ${y - 6}Q${x} ${y + ry * 0.62} ${x + rx * 0.8} ${y - 6}`}
          fill="none"
          stroke="#2a0d05"
          strokeWidth={22}
          strokeLinecap="round"
        />
        <path
          className="avatar-happy"
          d={`M${x - rx * 0.78} ${y + ry * 0.3}Q${x} ${y - ry * 1.3} ${x + rx * 0.78} ${y + ry * 0.3}`}
          fill="none"
          stroke="#2a0d05"
          strokeWidth={22}
          strokeLinecap="round"
        />
      </g>
    </g>
  );
}
