import type { ReactNode } from "react";
import { useT } from "../i18n";

/**
 * Technical text folded away under the plain message: an error's detail for
 * troubleshooting, or the command a bot wants to run. `label` names what is
 * inside when "Details" would not; `open` shows it from the start.
 */
export function Details({
  children,
  label,
  open = false,
}: {
  children: ReactNode;
  label?: string;
  open?: boolean;
}) {
  const t = useT();
  return (
    <details className="mt-2 text-xs" open={open}>
      <summary className="cursor-pointer text-muted hover:text-ink-soft">
        {label ?? t.common.details}
      </summary>
      <div className="mt-1 whitespace-pre-wrap break-words font-mono text-ink-soft" data-selectable>
        {children}
      </div>
    </details>
  );
}
