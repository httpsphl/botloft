import type { ReactNode } from "react";
import { useT } from "../i18n";

/** Technical text for troubleshooting, folded away under the plain message. */
export function Details({ children }: { children: ReactNode }) {
  const t = useT();
  return (
    <details className="mt-2 text-xs">
      <summary className="cursor-pointer text-muted hover:text-ink-soft">
        {t.common.details}
      </summary>
      <div className="mt-1 whitespace-pre-wrap break-words font-mono text-ink-soft" data-selectable>
        {children}
      </div>
    </details>
  );
}
