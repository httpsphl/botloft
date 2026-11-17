// The Maker's gear (spec 36.3), in the units of the mascot's artwork
// (mascotArt.ts). The Maker is Botloft's flame all in white (`whiten` in
// retint.ts), with a wand that floats by its right side with an orb of
// light at its tip and, behind it in the large sizes, a flow of cards (a
// crew, a routine, a flow) joined by dotted arrows.

/** The wand: its grip, how it leans at rest, its length, its orb and its colors. */
export const WAND = {
  grip: { x: 620, y: 610 },
  lean: 28,
  length: 232,
  orb: { y: -268, r: 26, glow: 70 },
  /** The glints around the orb, which twinkle while it works. */
  glints: [
    { x: -62, y: -320, r: 9 },
    { x: 58, y: -338, r: 7 },
    { x: -30, y: -372, r: 6 },
  ],
  colors: { stick: "#5A3A1E", band: "#FFC21A", core: "#FFF6D6", light: "#FFD08A", glow: "#FF7A59" },
};

/** The box the mascot takes with the wand beside it. */
export const WAND_BOX = [-10.75, 117.25, 834, 634] as const;

/** The box the mascot takes with its flow behind it. */
export const SCENE_BOX = [-720, -290, 2050, 1040] as const;

/** The flow behind the Maker: its cards, in order, and what each stands for. */
export const FLOW = {
  boxes: [
    { x: -470, y: 250 },
    { x: 196, y: -250 },
    { x: 860, y: 250 },
  ],
  box: { width: 230, height: 150, rx: 30 },
  /** What each card stands for: what the Maker builds. */
  steps: ["team", "routine", "flow"] as const,
};

/** The dotted arrow from one box to the next, bowing up between them. */
export function arrowPath(
  from: { x: number; y: number },
  to: { x: number; y: number },
  box: { width: number; height: number },
): string {
  const start = { x: from.x + box.width, y: from.y + box.height / 2 };
  const end = { x: to.x, y: to.y + box.height / 2 };
  const bend = { x: (start.x + end.x) / 2, y: Math.min(start.y, end.y) - 60 };
  return `M${start.x} ${start.y}Q${bend.x} ${bend.y} ${end.x - 24} ${end.y}`;
}
