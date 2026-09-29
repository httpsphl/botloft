// A screen drawn at its device's real size and scaled to fit (spec 22.5).
// Each new version loads in a hidden frame and takes the front once it is
// ready, so a screen being written grows without flickering. While the bot
// writes it, its cursor follows the end of the page and a frame marks the
// part being built (spec 22.3).

import { useEffect, useReducer, useRef, useState } from "react";
import type { Bot, ScreenDevice } from "../../lib/protocol.gen";
import { BotCursor } from "../bots/BotCursor";
import { type Mark, readMark } from "./cursorMark";

export const DEVICES: Record<ScreenDevice, { width: number; height: number }> = {
  desktop: { width: 1280, height: 800 },
  tablet: { width: 834, height: 1112 },
  mobile: { width: 390, height: 844 },
};

/** Where the cursor waits while nothing on the page shows yet. */
const PARKED = { x: 40, y: 40 };

/** The screen runs its own scripts, away from the app (spec 22.6). */
const SANDBOX = "allow-scripts allow-forms allow-popups allow-modals";

interface Frames {
  /** What each of the two frames holds. */
  urls: [string, string | null];
  /** The frame on show. */
  front: 0 | 1;
  /** The back frame is loading. */
  loading: boolean;
  /** The newest version, waiting for the back frame to finish. */
  waiting: string | null;
}

type Change = { kind: "show"; url: string } | { kind: "loaded"; frame: 0 | 1 };

function frames(state: Frames, change: Change): Frames {
  const back = state.front === 0 ? 1 : 0;
  if (change.kind === "show") {
    if (state.urls[state.front] === change.url) {
      return { ...state, waiting: null };
    }
    if (state.loading) {
      return { ...state, waiting: change.url };
    }
    const urls: Frames["urls"] = [...state.urls];
    urls[back] = change.url;
    return { ...state, urls, loading: true, waiting: null };
  }
  if (change.frame !== back || !state.loading) {
    return state;
  }
  const next: Frames = { ...state, front: back, loading: false };
  return state.waiting ? frames(next, { kind: "show", url: state.waiting }) : next;
}

export function LiveFrame({
  url,
  device,
  scale,
  title,
  interactive = false,
  writer = null,
}: {
  url: string;
  device: ScreenDevice;
  scale: number;
  title: string;
  /** Takes clicks and scrolling; otherwise the frame is only a picture. */
  interactive?: boolean;
  /** The bot writing it now, whose cursor shows. */
  writer?: Pick<Bot, "name" | "color"> | null;
}) {
  const [state, change] = useReducer(frames, {
    urls: [url, null],
    front: 0,
    loading: false,
    waiting: null,
  });
  useEffect(() => change({ kind: "show", url }), [url]);
  const size = DEVICES[device];
  const windows = useRef<(Window | null)[]>([null, null]);
  // Each frame's own mark: the one behind speaks before it is on show, and
  // its layout is the next version's, so only the front one's shows.
  const [marks, setMarks] = useState<[Mark | null, Mark | null]>([null, null]);
  const writing = writer !== null;
  useEffect(() => {
    if (!writing) {
      setMarks([null, null]);
      return;
    }
    const listen = (event: MessageEvent) => {
      const frame = event.source === null ? -1 : windows.current.indexOf(event.source as Window);
      const read = frame === 0 || frame === 1 ? readMark(event.data, size) : undefined;
      if (read) {
        setMarks((current) => (frame === 0 ? [read, current[1]] : [current[0], read]));
      }
    };
    window.addEventListener("message", listen);
    return () => window.removeEventListener("message", listen);
  }, [writing, size]);
  const mark = marks[state.front];
  const box = writing ? mark?.box : null;
  // Nothing to see yet (the styles come first): the cursor waits at the top.
  const point = writing && mark ? (mark.point ?? PARKED) : null;

  return (
    <div
      className="relative overflow-hidden bg-white"
      style={{ width: size.width * scale, height: size.height * scale }}
    >
      {([0, 1] as const).map((frame) => {
        const src = state.urls[frame];
        if (src === null) {
          return null;
        }
        const shown = frame === state.front;
        return (
          <iframe
            key={frame}
            ref={(node) => {
              windows.current[frame] = node?.contentWindow ?? null;
            }}
            src={src}
            title={title}
            sandbox={SANDBOX}
            aria-hidden={!shown}
            tabIndex={shown && interactive ? 0 : -1}
            onLoad={() => change({ kind: "loaded", frame })}
            className="absolute top-0 left-0 border-0 bg-white"
            style={{
              width: size.width,
              height: size.height,
              transform: `scale(${scale})`,
              transformOrigin: "0 0",
              visibility: shown ? "visible" : "hidden",
              pointerEvents: shown && interactive ? "auto" : "none",
            }}
          />
        );
      })}
      {box && writer && (
        <span
          aria-hidden
          className="pointer-events-none absolute rounded-[4px] border-2 transition-[left,top,width,height] duration-500 ease-out"
          style={{
            left: box.x * scale - 3,
            top: box.y * scale - 3,
            width: box.width * scale + 6,
            height: box.height * scale + 6,
            borderColor: writer.color,
            boxShadow: `0 0 0 4px ${writer.color}26`,
          }}
        />
      )}
      {point && writer && (
        <BotCursor
          bot={writer}
          left={`${point.x * scale}px`}
          top={`${point.y * scale}px`}
          typing
          flip={point.x > size.width * 0.7}
          lift={point.y > size.height * 0.88}
        />
      )}
    </div>
  );
}
