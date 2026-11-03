// The Maker's gear, drawn in the mascot's own SVG (spec 36.3, makerArt.ts):
// by its white flame, the wand from 72 px, and the flow behind it where
// there is room. Asleep, the wand lies down and the flow dims
// (mascot-maker.css);
// working, the wand waves, its glints twinkle and it throws sparks of
// light, and the flow's boxes light one by one, stepped with the flame
// (ticks.ts); cheering, the wand twirls and every box lights at once
// (gestures.ts).

import { arrowPath, FLOW, WAND } from "./makerArt";

/** The wand with its orb of light: upright and still, lying down asleep, waving at work. */
export function MakerWand({ glowId }: { glowId: string }) {
  const { grip, lean, length, orb, glints, colors } = WAND;
  return (
    <g className="maker-wand maker-wand-pose">
      <defs>
        <radialGradient id={glowId}>
          <stop offset="0" stopColor={colors.core} />
          <stop offset="0.55" stopColor={colors.light} />
          <stop offset="1" stopColor={colors.glow} stopOpacity={0} />
        </radialGradient>
      </defs>
      <g transform={`translate(${grip.x} ${grip.y})`}>
        <g transform={`rotate(${lean})`}>
          <g data-tick="wave">
            <circle cy={orb.y} r={orb.glow} fill={`url(#${glowId})`} />
            <line y2={-length} stroke={colors.stick} strokeWidth={18} strokeLinecap="round" />
            <line y1={-36} y2={-58} stroke={colors.band} strokeWidth={22} />
            <circle className="maker-orb" cy={orb.y} r={orb.r} fill={colors.core} />
            {glints.map((glint, index) => (
              <circle
                key={glint.x}
                data-tick="dot"
                data-phase={index}
                cx={glint.x}
                cy={glint.y}
                r={glint.r}
                fill={colors.light}
              />
            ))}
            {[0, 1].map((phase) => (
              <g key={phase} transform={`translate(0 ${orb.y})`}>
                <circle
                  className="maker-spark"
                  data-tick="fling"
                  data-phase={phase}
                  r={12}
                  fill={colors.light}
                  opacity={0}
                />
              </g>
            ))}
          </g>
        </g>
      </g>
    </g>
  );
}

/** A plain line icon of what the Maker builds, about 120 units across, centred on 0,0. */
function StepIcon({ kind }: { kind: "team" | "routine" | "flow" }) {
  if (kind === "team") {
    return (
      <>
        <circle cx={-22} cy={-18} r={18} />
        <path d="M-56 34C-52 8 8 8 12 34" />
        <circle cx={28} cy={-24} r={15} />
        <path d="M22 4C38 2 58 12 60 30" />
      </>
    );
  }
  if (kind === "routine") {
    return (
      <>
        <circle r={40} />
        <path d="M0 -22V0L16 12" />
      </>
    );
  }
  return (
    <>
      <circle cx={-34} r={12} />
      <circle cx={30} cy={-28} r={12} />
      <circle cx={30} cy={28} r={12} />
      <path d="M-22 -4L18 -24M-22 4L18 24" />
    </>
  );
}

/**
 * The flow behind the Maker: cards of what it builds (a crew, a routine, a
 * flow) joined by dotted arrows. A card lit in its turn rises a little and
 * its strip and icon take the accent (mascot-maker.css).
 */
export function MakerFlow({ arrowId, liftId }: { arrowId: string; liftId: string }) {
  const { boxes, box, steps } = FLOW;
  return (
    <g className="maker-flow">
      <defs>
        <marker
          id={arrowId}
          viewBox="0 0 10 10"
          refX="5"
          refY="5"
          markerWidth="3"
          markerHeight="3"
          orient="auto-start-reverse"
        >
          <path d="M0 0L10 5L0 10Z" fill="var(--line-strong)" />
        </marker>
        <filter id={liftId} x="-30%" y="-30%" width="160%" height="170%">
          <feDropShadow dx={0} dy={10} stdDeviation={12} floodColor="#000" floodOpacity={0.14} />
        </filter>
      </defs>
      {boxes.slice(1).map((to, index) => (
        <path
          key={to.x}
          className="maker-arrow"
          data-tick="run"
          d={arrowPath(boxes[index] ?? to, to, box)}
          fill="none"
          stroke="var(--line-strong)"
          strokeWidth={14}
          strokeLinecap="round"
          strokeDasharray="2 30"
          markerEnd={`url(#${arrowId})`}
        />
      ))}
      {boxes.map((at, index) => (
        <g key={at.x} transform={`translate(${at.x} ${at.y})`}>
          <g
            className="maker-box"
            data-tick="light"
            data-phase={index}
            data-count={boxes.length}
            filter={`url(#${liftId})`}
          >
            <rect width={box.width} height={box.height} rx={box.rx} fill="var(--panel)" />
            <path
              className="maker-strip"
              d={`M${box.rx} 0H${box.width - box.rx}A${box.rx} ${box.rx} 0 0 1 ${box.width} ${box.rx}V34H0V${box.rx}A${box.rx} ${box.rx} 0 0 1 ${box.rx} 0Z`}
              fill="var(--line-strong)"
            />
            <g
              className="maker-icon"
              transform={`translate(${box.width / 2} ${box.height / 2 + 16})`}
              fill="none"
              stroke="var(--muted)"
              strokeWidth={14}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <StepIcon kind={steps[index] ?? "flow"} />
            </g>
          </g>
        </g>
      ))}
    </g>
  );
}
