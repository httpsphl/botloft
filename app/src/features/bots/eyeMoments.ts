// When the mascots blink and glance around (spec 15.3). An animation that
// runs forever makes the WebView draw every frame, so the eyes do not run
// one: every couple of seconds this picks which mascots on screen blink,
// each a little apart, and now and then one idle mascot looks around.
// Between those moments the eyes stand still and cost nothing. Paused while
// the window is out of sight or has no focus, and with less motion.

/** How often, on average, each mascot blinks. */
const BLINK_EVERY_MS = 7000;
/** How often the scheduler looks; blinks of one look start together. */
const TICK_MS = 2000;
/** How far apart blinks of the same look may start. */
const SPREAD_MS = 600;
/** A blink, as `eye-blink` in mascot.css; a tired droop takes longer. */
const BLINK_MS = 260;
const DROOP_MS = 800;
/** One idle mascot looks around, at most this often, for this long. */
const LOOK_EVERY_MS = [9000, 16000] as const;
const LOOK_MS = 4400;

function resting(): boolean {
  const root = document.documentElement;
  return (
    document.visibilityState === "hidden" ||
    root.hasAttribute("data-hidden") ||
    root.hasAttribute("data-blurred") ||
    root.getAttribute("data-motion") === "less" ||
    (typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches)
  );
}

/** The mascots that may move their eyes: on screen, awake, not held still. */
function awake(): SVGElement[] {
  const all = document.querySelectorAll<SVGElement>(
    'svg[data-mood]:not([data-mood="sleeping"]):not([data-still])',
  );
  return [...all].filter((svg) => {
    const box = svg.getBoundingClientRect();
    return box.width > 0 && box.bottom > 0 && box.top < innerHeight;
  });
}

/** Sets `attribute` on `svg` for `ms`, after `delay`. */
function moment(svg: SVGElement, attribute: string, delay: number, ms: number): void {
  setTimeout(() => {
    svg.setAttribute(attribute, "");
    setTimeout(() => svg.removeAttribute(attribute), ms);
  }, delay);
}

/** Starts the eyes' moments; returns what stops them. */
export function startEyeMoments(random: () => number = Math.random): () => void {
  let nextLook = performance.now() + LOOK_EVERY_MS[0];
  const tick = () => {
    if (resting()) {
      return;
    }
    const mascots = awake();
    for (const svg of mascots) {
      if (random() < TICK_MS / BLINK_EVERY_MS) {
        const tired = svg.getAttribute("data-mood") === "tired";
        moment(svg, "data-blink", random() * SPREAD_MS, tired ? DROOP_MS : BLINK_MS);
      }
    }
    const now = performance.now();
    const idle = mascots.filter((svg) => svg.getAttribute("data-mood") === "idle");
    if (now >= nextLook && idle.length > 0) {
      const svg = idle[Math.floor(random() * idle.length)] as SVGElement;
      moment(svg, "data-look", 0, LOOK_MS);
      const [least, most] = LOOK_EVERY_MS;
      nextLook = now + least + random() * (most - least);
    }
  };
  const timer = setInterval(tick, TICK_MS);
  return () => clearInterval(timer);
}
