// The small drawing by the mascot's head that says what its bot is doing
// (mascotLook.ts): a round badge with a radar, a globe, a pencil, a
// terminal, a lens, a cursor, three dots, or a "!" while it waits for the
// owner. What moves in it is marked `data-tick` and stepped by the flame's
// timer (flame.ts), which runs anyway while the bot works: no animation of
// its own runs on and on. The "!" drops in once, with GSAP.

import { gsap } from "gsap";
import { useLayoutEffect, useRef } from "react";
import { lessMotion } from "./eyeMoments";
import type { Doing } from "./mascotLook";

/** Where the badge sits, in the artwork's units: up by the right of the head. */
export const BADGE = { x: 540, y: 250, r: 118 };

const LINE = {
  fill: "none",
  stroke: "var(--ink)",
  strokeWidth: 18,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

function Radar() {
  return (
    <>
      <circle r={62} {...LINE} />
      <circle r={30} {...LINE} strokeOpacity={0.5} />
      <g data-tick="spin">
        <path d="M0 0L0 -62A62 62 0 0 1 53.7 -31Z" fill="var(--work)" fillOpacity={0.55} />
        <line x2={0} y2={-62} {...LINE} stroke="var(--work)" />
      </g>
    </>
  );
}

function Globe() {
  return (
    <>
      <circle r={62} {...LINE} />
      <line x1={-62} x2={62} {...LINE} strokeOpacity={0.6} />
      <ellipse data-tick="turn" data-rx={28} rx={28} ry={62} {...LINE} stroke="var(--work)" />
    </>
  );
}

function Pencil() {
  return (
    <g data-tick="wiggle">
      <g transform="rotate(45)">
        <rect x={-16} y={-70} width={32} height={96} rx={6} {...LINE} />
        <path d="M-16 26L0 62L16 26" {...LINE} />
        <line x1={-16} y1={-46} x2={16} y2={-46} {...LINE} stroke="var(--work)" />
      </g>
    </g>
  );
}

function Terminal() {
  return (
    <>
      <rect x={-66} y={-50} width={132} height={100} rx={16} {...LINE} />
      <path d="M-38 -18L-14 2L-38 22" {...LINE} />
      <line data-tick="blink" x1={2} y1={26} x2={34} y2={26} {...LINE} stroke="var(--work)" />
    </>
  );
}

function Lens() {
  return (
    <g data-tick="scan">
      <circle cx={-12} cy={-12} r={36} {...LINE} />
      <line x1={14} y1={14} x2={50} y2={50} {...LINE} />
    </g>
  );
}

function Cursor() {
  return (
    <g data-tick="scan">
      <path d="M-30 -52L-30 34L-8 14L8 50L24 42L8 8L36 6Z" {...LINE} fill="var(--panel)" />
    </g>
  );
}

function Dots() {
  return (
    <>
      {[-40, 0, 40].map((x, index) => (
        <circle key={x} data-tick="dot" data-phase={index} cx={x} r={15} fill="var(--ink)" />
      ))}
    </>
  );
}

/** A "!" that drops in, bouncing, and sways to a stop. */
function Alert() {
  const mark = useRef<SVGGElement>(null);
  useLayoutEffect(() => {
    if (!mark.current || lessMotion()) return;
    const tween = gsap
      .timeline()
      .from(mark.current, { y: -160, opacity: 0, duration: 0.55, ease: "bounce.out" })
      .from(mark.current, { rotation: 18, duration: 0.9, ease: "elastic.out(1.2, 0.3)" }, 0.4);
    return () => {
      tween.kill();
    };
  }, []);
  return (
    <g ref={mark}>
      <rect x={-13} y={-62} width={26} height={78} rx={13} fill="var(--warn)" />
      <circle cy={46} r={15} fill="var(--warn)" />
    </g>
  );
}

const DRAWINGS: Record<Doing, () => React.JSX.Element> = {
  search: Radar,
  browse: Globe,
  write: Pencil,
  run: Terminal,
  read: Lens,
  control: Cursor,
  think: Dots,
  ask: Alert,
};

/** The badge for `doing`, drawn in the mascot's own SVG. */
export function MascotBadge({ doing }: { doing: Doing }) {
  const Drawing = DRAWINGS[doing];
  return (
    <g className="avatar-badge" data-doing={doing} transform={`translate(${BADGE.x} ${BADGE.y})`}>
      <circle
        r={BADGE.r}
        fill="var(--panel)"
        stroke="var(--line-strong)"
        strokeWidth={1}
        vectorEffect="non-scaling-stroke"
      />
      <Drawing />
    </g>
  );
}
