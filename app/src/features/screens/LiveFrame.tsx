// A screen drawn at its device's real size and scaled to fit (spec 22.5).
// Each new version loads in a hidden frame and takes the front once it is
// ready, so a screen being written grows without flickering.

import { useEffect, useReducer } from "react";
import type { ScreenDevice } from "../../lib/protocol.gen";

export const DEVICES: Record<ScreenDevice, { width: number; height: number }> = {
  desktop: { width: 1280, height: 800 },
  tablet: { width: 834, height: 1112 },
  mobile: { width: 390, height: 844 },
};

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
}: {
  url: string;
  device: ScreenDevice;
  scale: number;
  title: string;
  /** Takes clicks and scrolling; otherwise the frame is only a picture. */
  interactive?: boolean;
}) {
  const [state, change] = useReducer(frames, {
    urls: [url, null],
    front: 0,
    loading: false,
    waiting: null,
  });
  useEffect(() => change({ kind: "show", url }), [url]);
  const size = DEVICES[device];

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
    </div>
  );
}
