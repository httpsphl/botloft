// The owner's hands on the live screen (spec 21.10): over the picture of the
// page, their mouse and keyboard become events for the bot's browser. A
// hidden text box takes the keys, so accents, input methods and pasting work
// as in any field.

import {
  type CompositionEvent,
  type FormEvent,
  type KeyboardEvent,
  type MouseEvent,
  useEffect,
  useRef,
} from "react";
import type { BrowserInput, MouseButton } from "../../lib/protocol.gen";

/** Keys that only change others; they travel as `modifiers`. */
const MODIFIER_KEYS = new Set([
  "Shift",
  "Control",
  "Alt",
  "AltGraph",
  "Meta",
  "OS",
  "CapsLock",
  "NumLock",
  "ScrollLock",
  "Fn",
]);
/** Keys still composing a character: the text comes when it is done. */
const COMPOSING = new Set(["Dead", "Process", "Unidentified"]);
const TEXT_MAX = 10_000;
const WHEEL_MAX = 10_000;
/** Pixels in a line and a page of the wheel, for mice that count those. */
const WHEEL_LINE = 40;
const WHEEL_PAGE = 800;
const BUTTONS: MouseButton[] = ["left", "middle", "right"];

interface Modifiable {
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}

/** Alt 1, Ctrl 2, Meta 4, Shift 8, as the daemon takes them. */
export function modifiersOf(event: Modifiable): number {
  return (
    (event.altKey ? 1 : 0) |
    (event.ctrlKey ? 2 : 0) |
    (event.metaKey ? 4 : 0) |
    (event.shiftKey ? 8 : 0)
  );
}

const clamp = (value: number, max: number) => Math.min(max, Math.max(0, value));

export function HandsLayer({
  label,
  keysLabel,
  width,
  height,
  send,
  onFocus,
}: {
  label: string;
  keysLabel: string;
  /** The page's size, in CSS pixels. */
  width: number;
  height: number;
  send(input: BrowserInput): void;
  /** Whether the keys go to the page. */
  onFocus(focused: boolean): void;
}) {
  const area = useRef<HTMLDivElement>(null);
  const keys = useRef<HTMLTextAreaElement>(null);
  const composing = useRef(false);
  // Moves wait for the next frame; only the newest one goes.
  const move = useRef<BrowserInput | null>(null);
  const tick = useRef<number | null>(null);

  const point = (event: { clientX: number; clientY: number }) => {
    const box = area.current?.getBoundingClientRect();
    if (!box || box.width === 0 || box.height === 0) {
      return { x: 0, y: 0 };
    }
    return {
      x: clamp(((event.clientX - box.left) / box.width) * width, width),
      y: clamp(((event.clientY - box.top) / box.height) * height, height),
    };
  };
  const flush = () => {
    if (tick.current !== null) {
      cancelAnimationFrame(tick.current);
      tick.current = null;
    }
    const pending = move.current;
    move.current = null;
    if (pending) {
      send(pending);
    }
  };
  // Anything but a move goes at once, after the move before it.
  const now = (input: BrowserInput) => {
    flush();
    send(input);
  };
  const mouse = (event: MouseEvent, action: "down" | "up") => {
    now({
      kind: "mouse",
      action,
      ...point(event),
      button: BUTTONS[event.button] ?? "left",
      buttons: event.buttons,
      clicks: Math.min(3, Math.max(1, event.detail)),
      modifiers: modifiersOf(event),
    });
  };
  const typed = (box: HTMLTextAreaElement) => {
    const text = box.value;
    box.value = "";
    if (text) {
      now({ kind: "text", text: text.slice(0, TEXT_MAX) });
    }
  };

  // The keys go to the page as soon as the owner takes it.
  useEffect(() => {
    keys.current?.focus({ preventScroll: true });
    return () => {
      if (tick.current !== null) {
        cancelAnimationFrame(tick.current);
      }
    };
  }, []);

  // The wheel scrolls the page, not the panel: that needs a listener that
  // is not passive.
  useEffect(() => {
    const node = area.current;
    if (!node) {
      return;
    }
    const wheel = (event: WheelEvent) => {
      event.preventDefault();
      const scale = event.deltaMode === 1 ? WHEEL_LINE : event.deltaMode === 2 ? WHEEL_PAGE : 1;
      const limit = (delta: number) => Math.max(-WHEEL_MAX, Math.min(WHEEL_MAX, delta * scale));
      now({
        kind: "wheel",
        ...point(event),
        dx: limit(event.deltaX),
        dy: limit(event.deltaY),
        modifiers: modifiersOf(event),
      });
    };
    node.addEventListener("wheel", wheel, { passive: false });
    return () => node.removeEventListener("wheel", wheel);
  });

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.nativeEvent.isComposing || COMPOSING.has(event.key) || MODIFIER_KEYS.has(event.key)) {
      return;
    }
    const altGr = event.ctrlKey && event.altKey;
    // Ctrl+V pastes the owner's own clipboard, through `paste`.
    if ((event.ctrlKey || event.metaKey) && !altGr && event.key.toLowerCase() === "v") {
      return;
    }
    event.preventDefault();
    now({ kind: "key", key: event.key, code: event.code, modifiers: modifiersOf(event) });
  };

  return (
    <div
      ref={area}
      role="application"
      aria-label={label}
      className="absolute inset-0 cursor-default touch-none select-none"
      onPointerDown={(event) => event.currentTarget.setPointerCapture?.(event.pointerId)}
      onMouseDown={(event) => {
        event.preventDefault();
        keys.current?.focus({ preventScroll: true });
        mouse(event, "down");
      }}
      onMouseUp={(event) => mouse(event, "up")}
      onMouseMove={(event) => {
        move.current = {
          kind: "mouse",
          action: "move",
          ...point(event),
          button: "none",
          buttons: event.buttons,
          clicks: 0,
          modifiers: modifiersOf(event),
        };
        if (tick.current === null) {
          tick.current = requestAnimationFrame(() => {
            tick.current = null;
            flush();
          });
        }
      }}
      onContextMenu={(event) => event.preventDefault()}
    >
      <textarea
        ref={keys}
        aria-label={keysLabel}
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
        className="sr-only"
        onKeyDown={onKeyDown}
        onPaste={(event) => {
          event.preventDefault();
          const text = event.clipboardData.getData("text/plain");
          if (text) {
            now({ kind: "text", text: text.slice(0, TEXT_MAX) });
          }
        }}
        onInput={(event: FormEvent<HTMLTextAreaElement>) => {
          if (!composing.current) {
            typed(event.currentTarget);
          }
        }}
        onCompositionStart={() => {
          composing.current = true;
        }}
        onCompositionEnd={(event: CompositionEvent<HTMLTextAreaElement>) => {
          composing.current = false;
          typed(event.currentTarget);
        }}
        onFocus={() => onFocus(true)}
        onBlur={() => onFocus(false)}
      />
    </div>
  );
}
