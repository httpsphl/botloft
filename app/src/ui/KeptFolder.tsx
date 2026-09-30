import type { ReactNode } from "react";

/** In a confirmation: what stays on disk, and the folder it is in. */
export function KeptFolder({ path, children }: { path: string; children: ReactNode }) {
  return (
    <div className="mt-3 rounded-lg bg-sunken px-3 py-2">
      <p>{children}</p>
      <p className="mt-1 break-all font-mono text-ink text-xs" data-selectable>
        {path}
      </p>
    </div>
  );
}
