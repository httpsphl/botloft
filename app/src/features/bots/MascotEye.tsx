// One of the mascot's eyes (BotAvatar.tsx): open with its glints, shut into
// a soft arc while asleep, or bent up into a smile while it cheers. Its
// face (mascot-look.css) and its gestures (gestures.ts) shape it.

import { type Ellipse, EYES, HIGHLIGHTS } from "./mascotArt";

/** The glints that sit on `eye`. */
function glintsOn(eye: Ellipse): Ellipse[] {
  const reach = Math.max(eye.rx, eye.ry) * 1.3;
  return HIGHLIGHTS.filter((glint) => Math.hypot(glint.x - eye.x, glint.y - eye.y) < reach);
}

const rotate = ({ x, y, turn }: Ellipse) => `rotate(${turn} ${x} ${y})`;

/**
 * How far the face leans, in degrees: the line through the two eyes, the
 * right one sitting higher. Shut and smiling eyes lean with it, and so
 * does the Maker's hat.
 */
function tiltOf(eyes: readonly Ellipse[]): number {
  const [left, right] = eyes;
  if (!left || !right) return 0;
  return (Math.atan2(right.y - left.y, right.x - left.x) * 180) / Math.PI;
}

export const TILT = tiltOf(EYES);

export function MascotEye({ eye, side, fill }: { eye: Ellipse; side: "l" | "r"; fill: string }) {
  const { x, y, rx, ry } = eye;
  return (
    <g>
      <g className={`avatar-eye avatar-eye-${side}`}>
        <ellipse cx={x} cy={y} rx={rx} ry={ry} transform={rotate(eye)} fill={fill} />
        <g className="avatar-hl">
          {glintsOn(eye).map((glint) => (
            <ellipse
              key={glint.x}
              cx={glint.x}
              cy={glint.y}
              rx={glint.rx}
              ry={glint.ry}
              transform={rotate(glint)}
              fill="#ffffff"
            />
          ))}
        </g>
      </g>
      {/* The arcs lean in a group of their own: a gesture's transform on
          the path would replace one set on the path itself. */}
      <g transform={`rotate(${TILT} ${x} ${y})`}>
        <path
          className="avatar-closed"
          d={`M${x - rx * 0.8} ${y - 6}Q${x} ${y + ry * 0.62} ${x + rx * 0.8} ${y - 6}`}
          fill="none"
          stroke="#2a0d05"
          strokeWidth={22}
          strokeLinecap="round"
        />
        <path
          className="avatar-happy"
          d={`M${x - rx * 0.78} ${y + ry * 0.3}Q${x} ${y - ry * 1.3} ${x + rx * 0.78} ${y + ry * 0.3}`}
          fill="none"
          stroke="#2a0d05"
          strokeWidth={22}
          strokeLinecap="round"
        />
      </g>
    </g>
  );
}
