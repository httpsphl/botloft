// A bot's cursor: an arrow in its color with its name, gliding to where it
// works. The browser shows it where the bot clicks (spec 21.8), the design
// area where the bot is writing a screen (spec 22.5).

import type { Bot } from "../../lib/protocol.gen";

/** Dark text on light colors, light text on dark ones. */
export function inkOn(color: string): string {
  const hex = color.replace("#", "");
  const [r, g, b] = [0, 2, 4].map((at) => Number.parseInt(hex.slice(at, at + 2), 16) / 255);
  const luminance = 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0);
  return luminance > 0.55 ? "#141414" : "#ffffff";
}

export function BotCursor({
  bot,
  left,
  top,
  typing = false,
  flip = false,
  lift = false,
}: {
  bot: Pick<Bot, "name" | "color">;
  /** Where the tip points, as CSS lengths inside the positioned parent. */
  left: string;
  top: string;
  /** Dots after the name: the bot is writing. */
  typing?: boolean;
  /** The name goes left of the arrow, near the right edge. */
  flip?: boolean;
  /** The name goes above the arrow, near the bottom edge. */
  lift?: boolean;
}) {
  return (
    <span
      aria-hidden
      className={`bot-cursor ${flip ? "flex-row-reverse" : ""}`}
      style={{ left, top, ...(flip && { translate: "calc(-100% + 18px) -3px" }) }}
    >
      <svg width="22" height="22" viewBox="0 0 24 24" className="shrink-0 drop-shadow-sm">
        <title>{bot.name}</title>
        <path
          d="M4 2.5 19.5 12l-7 1.6-3.4 6.9Z"
          fill={bot.color}
          stroke="#ffffff"
          strokeWidth="1.6"
          strokeLinejoin="round"
        />
      </svg>
      <span
        className={`${flip ? "mr-1" : "ml-3.5"} -mt-1 inline-flex items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 font-semibold text-[11px] shadow-sm`}
        style={{
          background: bot.color,
          color: inkOn(bot.color),
          ...(lift && { marginTop: "-20px" }),
        }}
      >
        {bot.name}
        {typing && <span className="bot-typing">•••</span>}
      </span>
    </span>
  );
}
