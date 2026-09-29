// The bot's screen, live (spec 21.8): the newest picture of its active
// tab, with its cursor gliding to each point it acts on, a ring where it
// clicks, and a line about what it just did.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserAction, BrowserFrame } from "../../lib/protocol.gen";

const PAGE = { width: 1280, height: 800 };

/** Dark text on light colors, light text on dark ones. */
function inkOn(color: string): string {
  const hex = color.replace("#", "");
  const [r, g, b] = [0, 2, 4].map((at) => Number.parseInt(hex.slice(at, at + 2), 16) / 255);
  const luminance = 0.2126 * (r ?? 0) + 0.7152 * (g ?? 0) + 0.0722 * (b ?? 0);
  return luminance > 0.55 ? "#141414" : "#ffffff";
}

/** Where on the screen, in percent, an action happened. */
function spot(action: BrowserAction | null, frame: BrowserFrame | null) {
  if (!action || action.x === null || action.y === null) {
    return null;
  }
  const width = frame?.width ?? PAGE.width;
  const height = frame?.height ?? PAGE.height;
  return {
    x: Math.min(100, Math.max(0, (action.x / width) * 100)),
    y: Math.min(100, Math.max(0, (action.y / height) * 100)),
  };
}

export function useCaption(bot: Bot, action: BrowserAction | null): string | null {
  const did = useT().browser.did;
  if (!action) {
    return null;
  }
  const label = action.label?.trim() || null;
  switch (action.kind) {
    case "open":
      return label ? did.open(label) : null;
    case "click":
      return label ? did.click(label) : did.clickSomewhere;
    case "type":
      return label ? did.type(label) : did.typeSomewhere;
    case "select":
      return label ? did.select(label) : did.clickSomewhere;
    case "press":
      return did.press(label ?? "");
    case "scroll":
      return did.scroll;
    case "back":
      return did.back;
  }
  return bot.name;
}

export function LiveView({
  bot,
  frame,
  action,
  dim = false,
}: {
  bot: Bot;
  frame: BrowserFrame | null;
  action: BrowserAction | null;
  /** The browser closed: the last picture stays, faded. */
  dim?: boolean;
}) {
  const t = useT();
  const [cursor, setCursor] = useState<{ x: number; y: number } | null>(null);
  const point = spot(action, frame);
  // biome-ignore lint/correctness/useExhaustiveDependencies: each action moves the cursor once
  useEffect(() => {
    if (point) {
      setCursor(point);
    }
  }, [action?.at, action?.kind]);
  const width = frame?.width ?? PAGE.width;
  const height = frame?.height ?? PAGE.height;
  const ink = inkOn(bot.color);

  return (
    <figure
      aria-label={t.browser.screen(bot.name)}
      className="relative m-0 w-full overflow-hidden rounded-xl border border-line bg-canvas shadow-sm"
      style={{ aspectRatio: `${width} / ${height}` }}
    >
      {frame && (
        <img
          src={`data:image/jpeg;base64,${frame.data}`}
          alt=""
          draggable={false}
          className={`absolute inset-0 h-full w-full select-none object-contain transition-[opacity,filter] duration-500 ${
            dim ? "opacity-40 grayscale" : ""
          }`}
        />
      )}
      {!dim && action?.kind === "click" && point && (
        <span
          key={action.at}
          aria-hidden
          className="browser-ripple"
          style={{ left: `${point.x}%`, top: `${point.y}%`, borderColor: bot.color }}
        />
      )}
      {!dim && cursor && (
        <span
          aria-hidden
          className="browser-cursor"
          style={{ left: `${cursor.x}%`, top: `${cursor.y}%` }}
        >
          <svg width="22" height="22" viewBox="0 0 24 24" className="drop-shadow-sm">
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
            className="ml-3.5 -mt-1 inline-flex items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 font-semibold text-[11px] shadow-sm"
            style={{ background: bot.color, color: ink }}
          >
            {bot.name}
            {action?.kind === "type" && <span className="browser-typing">•••</span>}
          </span>
        </span>
      )}
    </figure>
  );
}
