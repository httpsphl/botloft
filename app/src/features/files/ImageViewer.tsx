// A picture of the bot's, as big as the window allows (spec 15.1): it opens
// from the picture on its card and closes with Escape, its button or a
// click beside it. It sits on the page itself, out of the chat's moving
// parts, so nothing it is inside can hold it.

import { X } from "lucide-react";
import { useEffect, useId, useLayoutEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useT } from "../../i18n";
import { exitDialog } from "../../ui/dialogExit";

export function ImageViewer({
  src,
  name,
  onClose,
}: {
  src: string;
  name: string;
  onClose(): void;
}) {
  const t = useT();
  const titleId = useId();
  const backdrop = useRef<HTMLDivElement>(null);
  const close = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const node = backdrop.current;
    return () => exitDialog(node);
  }, []);
  useEffect(() => {
    // Back where the owner was when it closes.
    const before = document.activeElement as HTMLElement | null;
    close.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      before?.focus?.();
    };
  }, [onClose]);

  return createPortal(
    // A click on the dark around the picture closes it; Escape does too.
    // biome-ignore lint/a11y/useKeyWithClickEvents: Escape and the button close it from the keyboard
    <div
      ref={backdrop}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      onClick={(event) => event.target === event.currentTarget && onClose()}
      className="fixed inset-0 z-40 flex animate-fade flex-col bg-black/85 p-4"
    >
      <div className="flex shrink-0 items-center gap-3 pb-3 text-white">
        <h2 id={titleId} className="min-w-0 flex-1 truncate font-medium text-sm">
          <span className="sr-only">{t.files.picture.viewer(name)}</span>
          <span aria-hidden>{name}</span>
        </h2>
        <button
          ref={close}
          type="button"
          aria-label={t.common.close}
          title={t.common.close}
          onClick={onClose}
          className="grid size-9 place-items-center rounded-full text-white/80 transition-colors hover:bg-white/15 hover:text-white"
        >
          <X aria-hidden size={18} />
        </button>
      </div>
      {/* Clicks around the picture pass through to the dark behind it. */}
      <div className="pointer-events-none grid min-h-0 flex-1 animate-pop place-items-center">
        <img
          src={src}
          alt={name}
          className="pointer-events-auto max-h-full max-w-full object-contain"
        />
      </div>
    </div>,
    document.body,
  );
}
