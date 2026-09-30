import { type MouseEvent, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { type MenuItem, MenuItems, menuKeys } from "./Menu";
import { POPOVER } from "./surface";
import { useDismiss } from "./useDismiss";

/** A place in the window, in CSS pixels. */
export interface Point {
  x: number;
  y: number;
}

/** How close to the window's edge a menu may get. */
const EDGE = 8;

/**
 * Where a `contextmenu` event asks for its menu: at the pointer, or, from
 * the keyboard's menu key, which has none, just inside the element.
 */
export function menuPoint(event: MouseEvent<HTMLElement>): Point {
  if (event.clientX !== 0 || event.clientY !== 0) {
    return { x: event.clientX, y: event.clientY };
  }
  const box = event.currentTarget.getBoundingClientRect();
  return { x: box.left + 24, y: box.top + box.height / 2 };
}

/**
 * Window pixels, which the pointer is in, per pixel of the page. 1 in the
 * app, whose webview does the zoom; the preview zooms the page with CSS.
 */
function pageScale(): number {
  const page = document.documentElement;
  return (page.offsetWidth && page.getBoundingClientRect().width / page.offsetWidth) || 1;
}

/**
 * The menu of a right-click: the rows of `Menu`, opened at `at` and kept
 * inside the window. Closes on a pick, Escape, a click elsewhere, or when
 * what is under it moves.
 */
export function ContextMenu({
  label,
  items,
  at,
  onClose,
}: {
  label: string;
  items: MenuItem[];
  at: Point;
  onClose(): void;
}) {
  const root = useRef<HTMLDivElement>(null);
  // Measured and put in place before it is first painted.
  const [place, setPlace] = useState(at);
  useDismiss(true, root, onClose);

  useLayoutEffect(() => {
    const menu = root.current;
    if (!menu) {
      return;
    }
    // Its own size, not its box: it is still growing into place.
    const scale = pageScale();
    const width = menu.offsetWidth * scale;
    const height = menu.offsetHeight * scale;
    setPlace({
      x: Math.max(EDGE, Math.min(at.x, window.innerWidth - width - EDGE)) / scale,
      y: Math.max(EDGE, Math.min(at.y, window.innerHeight - height - EDGE)) / scale,
    });
    // The keyboard follows the menu, and goes back where it was after it.
    const before = document.activeElement;
    menu.querySelector<HTMLElement>('[role^="menuitem"]:not(:disabled)')?.focus();
    return () => {
      if (menu.contains(document.activeElement) && before instanceof HTMLElement) {
        before.focus();
      }
    };
  }, [at]);

  useLayoutEffect(() => {
    window.addEventListener("resize", onClose);
    window.addEventListener("blur", onClose);
    window.addEventListener("scroll", onClose, true);
    return () => {
      window.removeEventListener("resize", onClose);
      window.removeEventListener("blur", onClose);
      window.removeEventListener("scroll", onClose, true);
    };
  }, [onClose]);

  // In the body: no list that scrolls or moves can clip or carry it.
  return createPortal(
    <div
      ref={root}
      role="menu"
      aria-label={label}
      onKeyDown={menuKeys}
      onContextMenu={(event) => event.preventDefault()}
      style={{ left: place.x, top: place.y }}
      className={`fixed min-w-60 origin-top-left ${POPOVER}`}
    >
      <MenuItems items={items} onPick={onClose} />
    </div>,
    document.body,
  );
}
