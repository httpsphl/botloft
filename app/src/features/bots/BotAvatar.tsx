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
// Given a `mood`, the mascot comes alive (mascot.css): its flame burns,
// gently when idle, glancing around now and then, and wildly, throwing
// embers, while it works; it hops
// while it waits for the owner, burns low with heavy eyes when tired and
// sleeps with its eyes shut while paused. When it finishes what it was
// doing, it cheers once (mascot-cheer.css). Without one it stays still, as in
// the chat history.

import { type CSSProperties, useEffect, useId, useMemo, useState } from "react";
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
export const CHEER_MS = 1200;

/**
 * Whether the mascot cheers now: from the moment its mood goes from working
 * to idle, for CHEER_MS. One that shows up idle does not.
 */
function useCheer(mood: Mood | undefined): boolean {
  const [last, setLast] = useState(mood);
  const [cheer, setCheer] = useState(false);
  if (mood !== last) {
    setLast(mood);
    setCheer(last === "working" && mood === "idle");
  }
  useEffect(() => {
    if (!cheer) return;
    const timer = setTimeout(() => setCheer(false), CHEER_MS);
    return () => clearTimeout(timer);
  }, [cheer]);
  return cheer;
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
  still = false,
}: {
  color: string;
  size?: number;
  framed?: boolean;
  mood?: Mood | undefined;
  /** Shows the mood without moving. */
  still?: boolean;
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
  const cheer = useCheer(mood);
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
      data-still={still || undefined}
      data-cheer={cheer || undefined}
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
  );
}

/**
 * A bot's mascot in a list (the sidebar, a crew's cards): it moves while
 * the bot is awake, idle too, since the owner reads an idle bot by its
 * slow flame and glances. A sleeping one keeps its look but does not
 * move: a morphing flame repaints every frame, and a stopped bot has
 * nothing to show.
 */
export function ListAvatar({
  bot,
  crewPaused,
  size,
}: {
  bot: Pick<Bot, "color" | "state" | "paused">;
  crewPaused: boolean;
  size: number;
}) {
  const mood = moodOf(bot, crewPaused);
  const still = mood === "sleeping";
  return <BotAvatar color={bot.color} size={size} mood={mood} still={still} />;
}
