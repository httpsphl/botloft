// How a side panel slides (spec 15.1): open from the window's right edge,
// or from the width of the panel it replaces, and back to the edge before
// it goes. One that comes back with its view does not slide. The keyframes
// are in panels.css; reduced motion makes both instant, and a timer stands
// in when the animation never reports its end.

import {
  type AnimationEvent,
  createContext,
  type RefObject,
  useContext,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

export const OPEN_MS = 460;
export const CLOSE_MS = 360;
/** How close in time a panel going and one coming count as a switch. */
const SWITCH_MS = 120;

/** Given to a panel while it slides closed; `closed` lets it go. */
export const PanelClosing = createContext<{ closed(): void } | null>(null);

/**
 * True for a panel that comes back with its view, as the owner left it: it
 * is there at once, without sliding open.
 */
export const PanelRestored = createContext(false);

/**
 * The panel that just went away without sliding: another replaces it. Not
 * the same one mounted again (React's strict mode in dev does that).
 */
let replaced: { width: number; at: number; panel: RefObject<HTMLElement | null> } | null = null;

export type Motion = "opening" | "open" | "closing";

export function usePanelMotion(panel: RefObject<HTMLElement | null>, width: number) {
  const exit = useContext(PanelClosing);
  const closing = exit !== null;
  const restored = useContext(PanelRestored);
  const [motion, setMotion] = useState<Motion>(restored ? "open" : "opening");
  const [from, setFrom] = useState(0);
  const now = useRef({ motion, width, exit });
  now.current = { motion, width, exit };

  // Before the first paint: start from the panel this one replaces, and
  // leave this one's width for the next when it goes the same way.
  useLayoutEffect(() => {
    if (replaced && replaced.panel !== panel && performance.now() - replaced.at < SWITCH_MS) {
      setFrom(replaced.width);
    }
    replaced = null;
    const element = panel;
    return () => {
      if (now.current.motion !== "closing") {
        const shown = element.current?.offsetWidth || now.current.width;
        replaced = { width: shown, at: performance.now(), panel: element };
      }
    };
  }, [panel]);

  useEffect(() => {
    if (closing) {
      setMotion("closing");
      const gone = setTimeout(() => now.current.exit?.closed(), CLOSE_MS + 150);
      return () => clearTimeout(gone);
    }
    // Opening, or opened again while it was closing.
    setMotion((current) => (current === "open" ? "open" : "opening"));
    const opened = setTimeout(() => setMotion("open"), OPEN_MS + 150);
    return () => clearTimeout(opened);
  }, [closing]);

  const ended = (event: AnimationEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget) {
      return;
    }
    if (event.animationName === "panel-open") {
      setMotion((current) => (current === "opening" ? "open" : current));
    } else if (event.animationName === "panel-close") {
      now.current.exit?.closed();
    }
  };

  return { motion, from, ended };
}
