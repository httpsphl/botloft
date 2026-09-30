import type { ReactNode } from "react";

/**
 * In a confirmation: what happens to a folder on disk, and where it is.
 * `recycle` adds the choice to send it to the Recycle Bin.
 */
export function KeptFolder({
  path,
  children,
  recycle,
}: {
  path: string;
  children: ReactNode;
  recycle?: { label: string; checked: boolean; onChange(checked: boolean): void };
}) {
  return (
    <div className="mt-3 rounded-lg bg-sunken px-3 py-2">
      <p>{children}</p>
      <p className="mt-1 break-all font-mono text-ink text-xs" data-selectable>
        {path}
      </p>
      {recycle && (
        <label className="mt-2 flex items-center gap-2 border-line border-t pt-2 text-ink">
          <input
            type="checkbox"
            checked={recycle.checked}
            onChange={(event) => recycle.onChange(event.target.checked)}
          />
          {recycle.label}
        </label>
      )}
    </div>
  );
}
