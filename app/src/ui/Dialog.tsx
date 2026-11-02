import { X } from "lucide-react";
import { type ReactNode, useEffect, useId, useLayoutEffect, useRef } from "react";
import { useT } from "../i18n";
import { Button } from "./Button";
import { exitDialog } from "./dialogExit";

interface DialogProps {
  title: string;
  onClose(): void;
  children: ReactNode;
  /** Buttons along the bottom edge. */
  footer?: ReactNode;
  width?: "md" | "lg";
}

/**
 * A modal panel. Escape and the close button call `onClose`. It pops in,
 * and fades out however its owner closes it (dialogExit.ts).
 */
export function Dialog({ title, onClose, children, footer, width = "md" }: DialogProps) {
  const t = useT();
  const titleId = useId();
  const backdrop = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const node = backdrop.current;
    return () => exitDialog(node);
  }, []);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div
      ref={backdrop}
      className="fixed inset-0 z-40 grid animate-fade place-items-center bg-scrim p-6"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={`flex max-h-full w-full animate-pop flex-col overflow-hidden rounded-2xl border border-line bg-panel shadow-lift ${width === "lg" ? "max-w-2xl" : "max-w-md"}`}
      >
        {/* No line under the title: the space sets it apart, as in a sheet. */}
        <header className="flex h-14 shrink-0 items-center justify-between pr-3 pl-6">
          <h2 id={titleId} className="font-semibold text-lg tracking-tight">
            {title}
          </h2>
          <Button variant="ghost" icon={X} label={t.common.close} onClick={onClose} />
        </header>
        <div className="min-h-0 overflow-y-auto px-6 pt-1 pb-5">{children}</div>
        {footer && (
          <footer className="flex shrink-0 justify-end gap-2 border-line border-t px-6 py-3.5">
            {footer}
          </footer>
        )}
      </div>
    </div>
  );
}
