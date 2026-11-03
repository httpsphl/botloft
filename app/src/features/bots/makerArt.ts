// The Maker's gear (spec 36.3), in the units of the mascot's artwork
// (mascotArt.ts): a builder's hard hat with a gear on its front, sitting
// just above the eyes and leaning with them; a wand that floats by its
// right side, its tip a small ember; and, behind it in the large sizes, a
// flow of boxes joined by dotted arrows with a small flame at each end.
// The hat keeps its own builder's yellow: it is not re-tinted, so the
// Maker never looks like an orange agent.

/** Where the hat's brim meets the head: the hat leans about this point. */
export const HAT_AT = { x: 346, y: 410 };

export const HAT = {
  dome: "M178 412C178 300 252 268 346 268C440 268 514 300 514 412Z",
  ridge: "M330 274C336 272 356 272 362 274L364 412L328 412Z",
  brim: { x: 146, y: 398, width: 400, height: 38, rx: 19 },
  gear: { x: 268, y: 352, r: 30, teeth: 8 },
  colors: {
    dome: "#FFC21A",
    shade: "#E09A00",
    brim: "#F2AE00",
    rim: "#B57C00",
    light: "#FFE586",
    gear: "#8A5400",
  },
};

/** A gear of `teeth` teeth around (x, y), `r` to the root of the teeth. */
export function gearPath({ x, y, r, teeth }: typeof HAT.gear): string {
  const outer = r * 1.32;
  const points: string[] = [];
  for (let i = 0; i < teeth * 4; i++) {
    const angle = (Math.PI * 2 * i) / (teeth * 4) - Math.PI / 2;
    const radius = i % 4 === 1 || i % 4 === 2 ? outer : r;
    points.push(
      `${(x + Math.cos(angle) * radius).toFixed(1)} ${(y + Math.sin(angle) * radius).toFixed(1)}`,
    );
  }
  return `M${points.join("L")}Z`;
}

/** The wand: its grip, how it leans at rest, its length and its colors. */
export const WAND = {
  grip: { x: 660, y: 640 },
  lean: 18,
  length: 270,
  colors: { stick: "#5A3A1E", band: "#FFC21A", tipLight: "#FFD08A", tip: "#FF7A59" },
};

/** The box the mascot takes with the wand beside it. */
export const WAND_BOX = [-10.75, 117.25, 834, 634] as const;

/** The box the mascot takes with its flow behind it. */
export const SCENE_BOX = [-720, -150, 2050, 900] as const;

/** The flow behind the Maker: its boxes, in order, and where its flames sit. */
export const FLOW = {
  boxes: [
    { x: -470, y: 250 },
    { x: 196, y: -120 },
    { x: 860, y: 250 },
  ],
  box: { width: 230, height: 150, rx: 26 },
  flames: [
    { x: -620, y: 330 },
    { x: 1210, y: 330 },
  ],
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
