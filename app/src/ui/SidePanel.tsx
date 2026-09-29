// A panel beside the main area whose width the owner sets by dragging its
// left edge (or with the arrow keys), remembered per panel. It never takes
// more than the main area can spare, so the chat keeps its room. It slides
// open and closed at the right edge (panelMotion.ts).

import {
  type CSSProperties,
  type KeyboardEvent,
  type PointerEvent,
  type ReactNode,
  useRef,
  useState,
} from "react";
import { useT } from "../i18n";
import { usePanelMotion } from "./panelMotion";

const MIN_PX = 256;
const MAX_PX = 900;
const KEY_STEP_PX = 24;
/** What the main area keeps whatever the panel's width, in rem. */
const KEEP_FOR_MAIN = "22rem";

function stored(name: string, fallback: number): number {
  try {
    const value = Number(localStorage.getItem(`botloft.panel.${name}`));
    return Number.isFinite(value) && value >= MIN_PX ? Math.min(value, MAX_PX) : fallback;
  } catch {
    return fallback;
  }
}

function remember(name: string, width: number): void {
  try {
    localStorage.setItem(`botloft.panel.${name}`, String(Math.round(width)));
  } catch {
    // A convenience: the width just is not kept.
  }
}

const clamp = (width: number) => Math.min(MAX_PX, Math.max(MIN_PX, width));

export function SidePanel({
  label,
  name,
  defaultWidth,
  expanded = false,
  children,
}: {
  label: string;
  /** Which panel this is, for remembering its width. */
  name: string;
  /** Width in px until the owner drags it. */
  defaultWidth: number;
  /** Takes all the room the main area can spare, until turned off. */
  expanded?: boolean;
  children: ReactNode;
}) {
  const t = useT();
  const [width, setWidth] = useState(() => stored(name, defaultWidth));
  const panel = useRef<HTMLElement>(null);
  const drag = useRef<{ x: number; width: number; scale: number } | null>(null);
  const { motion, from, ended } = usePanelMotion(panel, width);
  // While it slides, the content keeps its full width and is cut at the
  // moving edge, so it comes and goes without reflowing.
  const sliding = motion !== "open";

  const set = (next: number, keep = false) => {
    const clamped = clamp(next);
    setWidth(clamped);
    if (keep) {
      remember(name, clamped);
    }
  };

  const start = (event: PointerEvent<HTMLDivElement>) => {
    const element = panel.current;
    if (!element) {
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    // The window may be drawn larger than its layout units (zoom).
    const scale = element.offsetWidth
      ? element.getBoundingClientRect().width / element.offsetWidth
      : 1;
    drag.current = { x: event.clientX, width: element.offsetWidth, scale: scale || 1 };
  };
  const move = (event: PointerEvent<HTMLDivElement>) => {
    const from = drag.current;
    if (from) {
      // The panel is on the right: dragging left makes it wider.
      set(from.width + (from.x - event.clientX) / from.scale);
    }
  };
  const end = () => {
    if (drag.current) {
      drag.current = null;
      remember(name, panel.current?.offsetWidth || width);
    }
  };
  const onKey = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? KEY_STEP_PX * 4 : KEY_STEP_PX;
    const current = panel.current?.offsetWidth || width;
    if (event.key === "ArrowLeft") {
      set(current + step, true);
    } else if (event.key === "ArrowRight") {
      set(current - step, true);
    } else {
      return;
    }
    event.preventDefault();
  };

  return (
    <aside
      ref={panel}
      aria-label={label}
      aria-hidden={motion === "closing" || undefined}
      inert={motion === "closing"}
      style={
        {
          width: expanded ? `calc(100% - ${KEEP_FOR_MAIN})` : width,
          maxWidth: `calc(100% - ${KEEP_FOR_MAIN})`,
          minWidth: MIN_PX,
          "--panel-from": `${from}px`,
        } as CSSProperties
      }
      className={`relative flex shrink-0 flex-col border-line border-l bg-panel ${
        motion === "opening" ? "panel-opening" : motion === "closing" ? "panel-closing" : ""
      }`}
      onAnimationEnd={ended}
    >
      {/* biome-ignore lint/a11y/useSemanticElements: a separator that can be dragged and focused is a widget, not an <hr> */}
      <div
        hidden={expanded}
        role="separator"
        aria-orientation="vertical"
        aria-label={t.common.resize}
        aria-valuemin={MIN_PX}
        aria-valuemax={MAX_PX}
        aria-valuenow={Math.round(width)}
        tabIndex={0}
        onPointerDown={start}
        onPointerMove={move}
        onPointerUp={end}
        onPointerCancel={end}
        onDoubleClick={() => set(defaultWidth, true)}
        onKeyDown={onKey}
        className="group absolute inset-y-0 -left-1 z-10 w-2 cursor-col-resize touch-none focus-visible:outline-none"
      >
        <span className="absolute inset-y-0 left-[3px] w-0.5 bg-transparent transition-colors group-hover:bg-line-strong group-focus-visible:bg-work group-active:bg-work" />
      </div>
      <div
        className="flex min-h-0 flex-1 flex-col"
        style={sliding && !expanded ? { width, alignSelf: "flex-start" } : undefined}
      >
        {children}
      </div>
    </aside>
  );
}
