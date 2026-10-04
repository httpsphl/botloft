// Runs in the reel page before anything else (scripts/reel.mjs reads it as
// text): the page's clock stands still until the recorder moves it, one
// frame at a time. Timers, animation frames, Date.now and performance.now
// follow it, and so do CSS animations and transitions, held paused at the
// time they have played to. The page then looks the same at a given time on
// any computer, however long each frame takes to capture.

(() => {
  const real = {
    setTimeout: window.setTimeout.bind(window),
    requestAnimationFrame: window.requestAnimationFrame.bind(window),
  };
  const epoch = Date.now();
  let now = 0;
  let nextId = 1;
  const timers = new Map();
  const frames = new Map();

  const delay = (ms) => Math.max(0, Number(ms) || 0);
  window.setTimeout = (run, ms, ...args) => {
    const id = nextId++;
    timers.set(id, { at: now + delay(ms), every: 0, run: () => run(...args) });
    return id;
  };
  window.setInterval = (run, ms, ...args) => {
    const id = nextId++;
    const every = Math.max(1, delay(ms));
    timers.set(id, { at: now + every, every, run: () => run(...args) });
    return id;
  };
  window.clearTimeout = (id) => timers.delete(id);
  window.clearInterval = (id) => timers.delete(id);
  window.requestAnimationFrame = (run) => {
    const id = nextId++;
    frames.set(id, run);
    return id;
  };
  window.cancelAnimationFrame = (id) => frames.delete(id);
  performance.now = () => now;
  Date.now = () => epoch + now;

  /** When each animation started, on this clock. */
  const started = new WeakMap();
  function hold() {
    for (const animation of document.getAnimations()) {
      if (!started.has(animation)) started.set(animation, now);
      animation.pause();
      animation.currentTime = now - started.get(animation);
    }
  }

  /** Lets the page's own work (React's renders) run in real time. */
  const settle = () => new Promise((done) => real.setTimeout(done, 0));

  /** Moves the clock `ms` on, firing what falls due on the way, in order. */
  window.__reelStep = async (ms) => {
    const until = now + ms;
    for (;;) {
      let next;
      for (const [id, timer] of timers) {
        if (timer.at <= until && (!next || timer.at < next.timer.at)) next = { id, timer };
      }
      if (!next) break;
      now = next.timer.at;
      if (next.timer.every) next.timer.at += next.timer.every;
      else timers.delete(next.id);
      try {
        next.timer.run();
      } catch (error) {
        console.error(error);
      }
      await settle();
    }
    now = until;
    const due = [...frames.values()];
    frames.clear();
    for (const run of due) run(now);
    await settle();
    await settle();
    hold();
    // A picture waits for the next painted frame, and a step where nothing
    // moved paints none: a pixel in the corner, too dark to see, flips
    // shade every step so there always is one.
    if (!tick.isConnected) document.body.append(tick);
    tick.style.background = tick.style.background === "rgb(5, 5, 5)" ? "#060606" : "#050505";
    await new Promise((done) => real.requestAnimationFrame(() => done()));
  };
  const tick = document.createElement("div");
  tick.style.cssText = "position: fixed; left: 0; top: 0; width: 1px; height: 1px;";
})();
