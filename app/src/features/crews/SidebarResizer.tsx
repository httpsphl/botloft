import { type KeyboardEvent, type PointerEvent, useRef } from "react";
import { useT } from "../../i18n";
import {
  SIDEBAR_MAX_PX,
  SIDEBAR_MIN_PX,
  sidebarWidth,
  useSidebarWidth,
} from "../../shell/sidebarWidth";

const KEY_STEP_PX = 24;

/**
 * The right edge of the crews list: drag it, or use the arrow keys, to make
 * the list wider or narrower; double click goes back to the usual width. The
 * parent is `relative`.
 */
export function SidebarResizer() {
  const t = useT();
  const width = useSidebarWidth();
  const drag = useRef<{ x: number; width: number; scale: number } | null>(null);

  const start = (event: PointerEvent<HTMLDivElement>) => {
    const list = event.currentTarget.parentElement;
    if (!list) {
      return;
    }
    event.currentTarget.setPointerCapture(event.pointerId);
    // The window may be drawn larger than its layout units (zoom).
    const scale = list.offsetWidth ? list.getBoundingClientRect().width / list.offsetWidth : 1;
    drag.current = { x: event.clientX, width: sidebarWidth.get(), scale: scale || 1 };
  };
  const move = (event: PointerEvent<HTMLDivElement>) => {
    const from = drag.current;
    if (from) {
      sidebarWidth.set(from.width + (event.clientX - from.x) / from.scale);
    }
  };
  const end = () => {
    if (drag.current) {
      drag.current = null;
      sidebarWidth.set(sidebarWidth.get(), true);
    }
  };
  const onKey = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? KEY_STEP_PX * 4 : KEY_STEP_PX;
    if (event.key === "ArrowRight") {
      sidebarWidth.set(sidebarWidth.get() + step, true);
    } else if (event.key === "ArrowLeft") {
      sidebarWidth.set(sidebarWidth.get() - step, true);
    } else {
      return;
    }
    event.preventDefault();
  };

  return (
    // biome-ignore lint/a11y/useSemanticElements: a separator that can be dragged and focused is a widget, not an <hr>
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={t.common.resize}
      aria-valuemin={SIDEBAR_MIN_PX}
      aria-valuemax={SIDEBAR_MAX_PX}
      aria-valuenow={width}
      tabIndex={0}
      onPointerDown={start}
      onPointerMove={move}
      onPointerUp={end}
      onPointerCancel={end}
      onDoubleClick={() => sidebarWidth.reset()}
      onKeyDown={onKey}
      className="group absolute inset-y-0 -right-1 z-30 w-2 cursor-col-resize touch-none focus-visible:outline-none"
    >
      <span className="absolute inset-y-0 right-[3px] w-0.5 bg-transparent transition-colors group-hover:bg-line-strong group-focus-visible:bg-work group-active:bg-work" />
    </div>
  );
}
