import type { ReactNode } from "react";

/** Technical text for troubleshooting, folded away under the plain message. */
export function Details({ children }: { children: ReactNode }) {
  return (
    <details className="mt-2 text-xs">
      <summary className="cursor-pointer text-muted hover:text-ink-soft">Details</summary>
      <div className="mt-1 whitespace-pre-wrap break-words font-mono text-ink-soft" data-selectable>
        {children}
      </div>
    </details>
  );
}
