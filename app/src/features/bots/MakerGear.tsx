// The Maker's gear, drawn in the mascot's own SVG (spec 36.3, makerArt.ts):
// the hard hat, the wand from 72 px, and the flow behind it where there is
// room. Asleep, the hat slips over the eyes, the wand lies down and the
// flow dims (mascot-maker.css); working, the wand waves and throws sparks
// and the flow's boxes light one by one, stepped with the flame (ticks.ts);
// cheering, the wand twirls and every box lights at once (gestures.ts).

import { arrowPath, FLOW, gearPath, HAT, HAT_AT, WAND } from "./makerArt";
import { EMBER } from "./mascotArt";

export function MakerHat({ tilt }: { tilt: number }) {
  const { brim, colors } = HAT;
  return (
    <g className="maker-hat">
      <g transform={`rotate(${tilt} ${HAT_AT.x} ${HAT_AT.y})`}>
        <path d={HAT.dome} fill={colors.dome} stroke={colors.rim} strokeWidth={8} />
        <path d={HAT.ridge} fill={colors.light} opacity={0.85} />
        <path
          d="M206 380C214 322 262 292 320 284"
          fill="none"
          stroke={colors.light}
          strokeWidth={14}
          strokeLinecap="round"
          opacity={0.8}
        />
        <path d={gearPath(HAT.gear)} fill={colors.gear} />
        <circle cx={HAT.gear.x} cy={HAT.gear.y} r={11} fill={colors.dome} />
        <rect {...brim} fill={colors.brim} stroke={colors.rim} strokeWidth={8} />
        <rect
          x={brim.x + 16}
          y={brim.y + brim.height - 12}
          width={brim.width - 32}
          height={8}
          rx={4}
          fill={colors.shade}
        />
      </g>
    </g>
  );
}

/** The wand: still and upright, lying down asleep, waving while it works. */
export function MakerWand({ ember }: { ember: string }) {
  const { grip, lean, length, colors } = WAND;
  return (
    <g className="maker-wand maker-wand-pose">
      <g transform={`translate(${grip.x} ${grip.y})`}>
        <g className="maker-wand-rest" transform={`rotate(${lean})`}>
          <g data-tick="wave">
            <line y2={-length} stroke={colors.stick} strokeWidth={22} strokeLinecap="round" />
            <line y1={-40} y2={-70} stroke={colors.band} strokeWidth={26} strokeLinecap="butt" />
            <g transform={`translate(0 ${-length - 24}) scale(1.5)`}>
              <path className="maker-tip" d={EMBER.d} fill={ember} />
            </g>
            {[0, 1].map((phase) => (
              <g key={phase} transform={`translate(0 ${-length - 30})`}>
                <path
                  className="maker-spark"
                  data-tick="fling"
                  data-phase={phase}
                  d={EMBER.d}
                  fill={colors.tipLight}
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

/** The flow behind the Maker: boxes joined by dotted arrows, a flame at each end. */
export function MakerFlow({ arrowId, ember }: { arrowId: string; ember: string }) {
  const { boxes, box, flames } = FLOW;
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
      {flames.map((at) => (
        <g key={at.x} transform={`translate(${at.x} ${at.y}) scale(2.4)`}>
          <path d={EMBER.d} fill={ember} />
        </g>
      ))}
    </g>
  );
}
