// The Botloft mascot in the bot's own color (spec 15.3). The color only
// tells bots apart; state is shown elsewhere with an icon and a label.
//
// The artwork (mascotArt.ts) is a flat body with soft shading layers clipped
// to its outline, re-tinted as a whole to the bot's color. A bot's avatar
// has no background: just the mascot, with a thin edge so light colors keep
// their shape on the light theme. `framed` puts the mascot on the black
// square of the app icon, for Botloft itself (the title bar, messages from
// the daemon), whose white mascot needs it.
//
// Given a `mood`, the mascot comes alive (mascot.css): its flame burns,
// gently when idle and wildly, throwing embers, while it works; it hops
// while it waits for the owner, burns low with heavy eyes when tired and
// sleeps with its eyes shut while paused. Without one it stays still, as in
// the chat history.

import { type CSSProperties, useId, useMemo } from "react";
import type { Bot } from "../../lib/protocol.gen";
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
import { retint } from "./retint";

export type Mood = "idle" | "working" | "waiting" | "tired" | "sleeping";

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

export function BotAvatar({
  color,
  size = 28,
  framed = false,
  mood,
}: {
  color: string;
  size?: number;
  framed?: boolean;
  mood?: Mood | undefined;
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
  const style = mood ? ({ "--blink-delay": blinkDelay(color) } as CSSProperties) : undefined;
  const box = framed ? [CENTER.x - 500, CENTER.y - 470, 1000, 1000] : VIEW_BOX;
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      viewBox={box.join(" ")}
      className={`shrink-0 overflow-visible ${framed ? "rounded-lg bg-[#0b0b0b]" : ""}`}
      data-mood={mood}
      style={style}
    >
      <defs>
        <clipPath id={id("clip")}>
          <path className="avatar-fire" d={OUTLINE} />
        </clipPath>
        <filter id={id("soft")} x="-10%" y="-10%" width="120%" height="120%">
          <feGaussianBlur stdDeviation="4" />
        </filter>
        <linearGradient id={id("ember")} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={paint.emberLight} />
          <stop offset="1" stopColor={paint.emberBase} />
        </linearGradient>
        <radialGradient id={id("eye")} cx="0.36" cy="0.95" r="0.75">
          <stop offset="0" stopColor={paint.glow} />
          <stop offset="0.3" stopColor="#3a0f05" />
          <stop offset="1" stopColor="#120503" />
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
              <path key={shade.fill} d={shade.d} fill={paint.shades[index]} />
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
        {EYES.map((eye, index) => (
          <Eye key={eye.x} eye={eye} side={index === 0 ? "l" : "r"} fill={`url(#${id("eye")})`} />
        ))}
      </g>
    </svg>
  );
}

/** One eye: open with its glints, or shut into a soft arc while asleep. */
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
      <path
        className="avatar-closed"
        d={`M${x - rx * 0.8} ${y - 6}Q${x} ${y + ry * 0.62} ${x + rx * 0.8} ${y - 6}`}
        fill="none"
        stroke="#2a0d05"
        strokeWidth={22}
        strokeLinecap="round"
      />
    </g>
  );
}
