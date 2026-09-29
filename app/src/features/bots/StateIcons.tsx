// Two state icons that move (motion.css), where a still lucide icon would
// say less: a ready bot pulses, a working one runs a line. Both keep the
// lucide look (24 grid, 2px stroke) so they sit with the other states.

import type { ComponentType } from "react";

export interface StateIconProps {
  size?: number;
  className?: string;
  "aria-hidden"?: boolean;
}

/** A state's icon: one of these, or any lucide icon. */
export type StateIcon = ComponentType<StateIconProps>;

/** "Ready": a dot with two rings spreading from it, one after the other. */
export function ReadyIcon({ size = 14, className = "" }: StateIconProps) {
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      className={className}
    >
      <circle className="state-ring" cx="12" cy="12" r="5" />
      <circle className="state-ring state-ring-late" cx="12" cy="12" r="5" />
      <circle cx="12" cy="12" r="5" fill="currentColor" stroke="none" />
    </svg>
  );
}

// The line of lucide's Activity, the icon this state had before it moved.
const PULSE =
  "M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2";

/** "Working": the pulse line, faint, with a bright stretch running along it. */
export function WorkingIcon({ size = 14, className = "" }: StateIconProps) {
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d={PULSE} opacity={0.3} />
      <path className="state-trace" d={PULSE} pathLength={100} />
    </svg>
  );
}
