// The bot's screen, live (spec 21.8): the newest picture of its active
// tab, as large as fits the panel, with its cursor gliding to each point it
// acts on, a ring where it clicks, and a line about what it just did.

import { type ReactNode, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserAction, BrowserFrame } from "../../lib/protocol.gen";
import { BotCursor } from "../bots/BotCursor";

/** The page until a picture says its size. */
const PAGE = { width: 1280, height: 800 };

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
  held = false,
  children,
}: {
  bot: Bot;
  frame: BrowserFrame | null;
  action: BrowserAction | null;
  /** The browser closed: the last picture stays, faded. */
  dim?: boolean;
  /** The owner has it (spec 21.10): the bot's cursor steps aside. */
  held?: boolean;
  /** On top of the picture: the owner's hands. */
  children?: ReactNode;
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
  const acting = !dim && !held;

  return (
    <figure
      aria-label={t.browser.screen(bot.name)}
      className={`relative mx-auto my-0 overflow-hidden rounded-xl border bg-canvas shadow-sm transition-[border-color,box-shadow] duration-200 ${
        held ? "border-accent ring-2 ring-accent/35" : "border-line"
      }`}
      // As large as fits its place (a size container), in the page's shape.
      style={{
        aspectRatio: `${width} / ${height}`,
        width: `min(100cqw, 100cqh * ${width} / ${height})`,
      }}
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
      {acting && action?.kind === "click" && point && (
        <span
          key={action.at}
          aria-hidden
          className="browser-ripple"
          style={{ left: `${point.x}%`, top: `${point.y}%`, borderColor: bot.color }}
        />
      )}
      {acting && cursor && (
        <BotCursor
          bot={bot}
          left={`${cursor.x}%`}
          top={`${cursor.y}%`}
          typing={action?.kind === "type"}
        />
      )}
      {children}
    </figure>
  );
}
