import { X } from "lucide-react";
import { type ReactNode, useEffect, useId } from "react";
import { useT } from "../i18n";
import { Button } from "./Button";

interface DialogProps {
  title: string;
  onClose(): void;
  children: ReactNode;
  /** Buttons along the bottom edge. */
  footer?: ReactNode;
  width?: "md" | "lg";
}

/** A modal panel. Escape and the close button call `onClose`. */
export function Dialog({ title, onClose, children, footer, width = "md" }: DialogProps) {
  const t = useT();
  const titleId = useId();
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
    <div className="fixed inset-0 z-40 grid animate-fade place-items-center bg-black/50 p-6">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={`flex max-h-full w-full animate-pop flex-col overflow-hidden rounded-2xl border border-line bg-panel shadow-lift ${width === "lg" ? "max-w-2xl" : "max-w-md"}`}
      >
        <header className="flex h-12 shrink-0 items-center justify-between border-line border-b pr-2 pl-5">
          <h2 id={titleId} className="font-semibold text-base tracking-tight">
            {title}
          </h2>
          <Button variant="ghost" icon={X} label={t.common.close} onClick={onClose} />
        </header>
        <div className="min-h-0 overflow-y-auto px-5 py-4">{children}</div>
        {footer && (
          <footer className="flex shrink-0 justify-end gap-2 border-line border-t px-5 py-3">
            {footer}
          </footer>
        )}
      </div>
    </div>
  );
}
