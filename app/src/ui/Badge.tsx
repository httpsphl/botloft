// Small labels (spec 15.3): a pill with a word and maybe an icon, and the
// round count beside a crew, the question box and the crews' cards.

import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { TONES, type Tone } from "./tone";

/** A word in a soft pill of its tone, like "Chief" or "Closed". */
export function Badge({
  tone,
  icon: Icon,
  title,
  children,
}: {
  tone: Tone;
  icon?: LucideIcon;
  title?: string;
  children: ReactNode;
}) {
  return (
    <span
      title={title}
      className={`inline-flex shrink-0 items-center gap-1 rounded-full px-2 py-0.5 font-medium text-xs ${TONES[tone].soft}`}
    >
      {Icon && <Icon aria-hidden size={12} />}
      {children}
    </span>
  );
}

/** A number in a solid circle; `label` says it in words for screen readers. */
export function CountBadge({ tone, count, label }: { tone: Tone; count: number; label: string }) {
  return (
    <span
      className={`grid h-[18px] min-w-[18px] place-items-center rounded-full px-1 font-semibold text-[11px] tabular-nums ${TONES[tone].solid}`}
    >
      <span aria-hidden>{count}</span>
      <span className="sr-only">{label}</span>
    </span>
  );
}
