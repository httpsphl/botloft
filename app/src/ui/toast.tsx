// Short notices for actions that failed away from a form.

import { X } from "lucide-react";
import { useStore } from "zustand";
import { createStore } from "zustand/vanilla";
import { useT } from "../i18n";
import { errorText } from "../lib/api";

interface Toast {
  id: number;
  text: string;
}

const toasts = createStore<{ items: Toast[] }>(() => ({ items: [] }));
let nextId = 1;
const LIFETIME_MS = 8000;

export function dismiss(id: number): void {
  toasts.setState((state) => ({ items: state.items.filter((toast) => toast.id !== id) }));
}

export function notifyError(what: string, error: unknown): void {
  const id = nextId++;
  const text = `${what}: ${errorText(error)}`;
  toasts.setState((state) => ({ items: [...state.items.slice(-3), { id, text }] }));
  setTimeout(() => dismiss(id), LIFETIME_MS);
}

/** Runs `action`, reporting a failure as a toast. */
export async function attempt(what: string, action: () => Promise<unknown>): Promise<boolean> {
  try {
    await action();
    return true;
  } catch (error) {
    notifyError(what, error);
    return false;
  }
}

export function Toaster() {
  const t = useT();
  const items = useStore(toasts, (state) => state.items);
  return (
    <div className="pointer-events-none fixed right-4 bottom-4 z-50 flex w-96 flex-col gap-2">
      {items.map((toast) => (
        <div
          key={toast.id}
          role="alert"
          className="pointer-events-auto flex items-start gap-2 rounded-xl border border-danger/45 bg-panel py-2.5 pr-2 pl-3.5 text-sm shadow-lift"
        >
          <p className="min-w-0 flex-1 break-words" data-selectable>
            {toast.text}
          </p>
          <button
            type="button"
            aria-label={t.common.dismiss}
            onClick={() => dismiss(toast.id)}
            className="grid h-6 w-6 shrink-0 place-items-center rounded-md text-muted hover:bg-sunken hover:text-ink"
          >
            <X aria-hidden size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}
