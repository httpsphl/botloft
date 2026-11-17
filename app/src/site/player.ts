// Plays the site's scenes (spec 39.4): the demos of the app drawn in the
// page. A scene (`data-scene`) moves one step per beat while it is on
// screen, holds at its last step and starts again. Its parts say when they
// show: `data-at="n"` from step n on, `data-until="n"` until step n. A part
// with `data-type` writes its text out when it shows, and a mascot's place
// with `data-moods="0:awake,3:working"` changes mood on those steps.
//
// The built page shows every scene at its end, which is also what a
// visitor who asks for less motion sees.

export interface SceneTiming {
  /** How many steps, counting the first (0). */
  steps: number;
  /** Milliseconds per step. */
  beat: number;
  /** Milliseconds at the last step before it starts again. */
  hold: number;
}

/** Whether a part with these marks shows at `step`. */
export function shownAt(step: number, at: number | null, until: number | null): boolean {
  return (at === null || step >= at) && (until === null || step < until);
}

/** The mood a mascot has at `step`, from `0:awake,3:working`; null when none is due yet. */
export function moodAt(moods: string, step: number): string | null {
  let current: string | null = null;
  for (const entry of moods.split(",")) {
    const [from, mood] = entry.split(":");
    const at = Number(from);
    if (mood && Number.isFinite(at) && at <= step) {
      current = mood.trim();
    }
  }
  return current;
}

/** The scene's timing from its attributes, with safe defaults. */
export function timingOf(scene: HTMLElement): SceneTiming {
  const number = (value: string | undefined, fallback: number) => {
    const read = Number(value);
    return Number.isFinite(read) && read > 0 ? read : fallback;
  };
  return {
    steps: number(scene.dataset.steps, 1),
    beat: number(scene.dataset.beat, 1400),
    hold: number(scene.dataset.hold, 3200),
  };
}

const marked = (value: string | undefined) => (value === undefined ? null : Number(value));

/** Shows the scene as it is at `step`; `typing` writes out the parts that just showed. */
export function showStep(scene: HTMLElement, step: number, typing: (part: HTMLElement) => void) {
  scene.dataset.step = String(step);
  for (const part of scene.querySelectorAll<HTMLElement>("[data-at], [data-until]")) {
    const on = shownAt(step, marked(part.dataset.at), marked(part.dataset.until));
    const was = part.classList.contains("is-on");
    part.classList.toggle("is-on", on);
    if (on && !was && part.dataset.type !== undefined) {
      typing(part);
    }
  }
  for (const place of scene.querySelectorAll<HTMLElement>("[data-moods]")) {
    const mood = moodAt(place.dataset.moods ?? "", step);
    if (mood && mood !== place.dataset.mood) {
      place.dataset.mood = mood;
      place.dispatchEvent(new CustomEvent("site:mood", { detail: mood }));
    }
  }
}

/** Writes a part's text out over `ms`, a few letters at a time. */
function typeOut(part: HTMLElement, ms: number): () => void {
  const text = part.dataset.text ?? part.textContent ?? "";
  part.dataset.text = text;
  const started = performance.now();
  let frame = 0;
  const tick = () => {
    const share = Math.min(1, (performance.now() - started) / ms);
    part.textContent = text.slice(0, Math.ceil(text.length * share));
    if (share < 1) frame = requestAnimationFrame(tick);
  };
  tick();
  return () => {
    cancelAnimationFrame(frame);
    part.textContent = text;
  };
}

/**
 * Plays every scene under `root` while it is on screen. With `still`, each
 * stays at its last step. Returns a function that stops them all.
 */
export function playScenes(root: ParentNode, still: boolean): () => void {
  const stops: (() => void)[] = [];
  for (const scene of root.querySelectorAll<HTMLElement>("[data-scene]")) {
    const timing = timingOf(scene);
    const last = timing.steps - 1;
    if (still) {
      showStep(scene, last, () => {});
      continue;
    }
    let step = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let visible = false;
    const typed: (() => void)[] = [];
    const typing = (part: HTMLElement) => typed.push(typeOut(part, timing.beat * 0.8));
    const next = () => {
      step = step >= last ? 0 : step + 1;
      if (step === 0) {
        for (const finish of typed.splice(0)) finish();
      }
      showStep(scene, step, typing);
      timer = setTimeout(next, step === last ? timing.hold : timing.beat);
    };
    const run = (on: boolean) => {
      if (on === visible) return;
      visible = on;
      clearTimeout(timer);
      if (on) timer = setTimeout(next, timing.beat);
    };
    showStep(scene, 0, typing);
    if (typeof IntersectionObserver === "undefined") {
      run(true);
    } else {
      const watch = new IntersectionObserver((entries) => {
        for (const entry of entries) run(entry.isIntersecting);
      });
      watch.observe(scene);
      stops.push(() => watch.disconnect());
    }
    stops.push(() => {
      clearTimeout(timer);
      for (const finish of typed.splice(0)) finish();
    });
  }
  return () => {
    for (const stop of stops) stop();
  };
}
