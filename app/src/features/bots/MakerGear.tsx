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

/** The flow behind the Maker: boxes joined by dotted arrows. */
export function MakerFlow({ arrowId }: { arrowId: string }) {
  const { boxes, box } = FLOW;
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
          <path d="M0 0L10 5L0 10Z" fill="var(--accent)" />
        </marker>
      </defs>
      {boxes.slice(1).map((to, index) => (
        <path
          key={to.x}
          className="maker-arrow"
          data-tick="run"
          d={arrowPath(boxes[index] ?? to, to, box)}
          fill="none"
          stroke="var(--accent)"
          strokeWidth={16}
          strokeLinecap="round"
          strokeDasharray="2 30"
          markerEnd={`url(#${arrowId})`}
        />
      ))}
      {boxes.map((at, index) => (
        <g key={at.x} transform={`translate(${at.x} ${at.y})`}>
          <g className="maker-box" data-tick="light" data-phase={index} data-count={boxes.length}>
            <rect {...box} fill="var(--panel)" stroke="var(--accent)" strokeWidth={14} />
            <line
              x1={40}
              y1={58}
              x2={190}
              y2={58}
              stroke="var(--accent)"
              strokeWidth={14}
              strokeLinecap="round"
            />
            <line
              x1={40}
              y1={96}
              x2={140}
              y2={96}
              stroke="var(--accent)"
              strokeWidth={14}
              strokeLinecap="round"
              opacity={0.6}
            />
          </g>
        </g>
      ))}
    </g>
  );
}
