// Records the mascot reel (src/dev/Reel.tsx) to a video: opens the page in
// a headless Edge or Chrome with its clock stopped (reelClock.js), moves it
// a sixtieth of a second at a time and takes a picture of each frame over the
// DevTools protocol, then has ffmpeg (on the PATH) join them at 60 frames a
// second. The frames come out the same however slow the computer is. Run
// `pnpm dev` first.
//
//   pnpm reel                      12 s at 1920x1080 into .dev/reel/reel.mp4
//   pnpm reel --seconds 30 --size 1080x1920 --beat 2600
//   pnpm reel --blur               blends each frame with its neighbors
//   pnpm reel --url http://localhost:1456 --out clip.mp4
//
// REEL_BROWSER points to another Chromium browser.

import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const app = join(dirname(fileURLToPath(import.meta.url)), "..");

function option(name, fallback) {
  const at = process.argv.indexOf(`--${name}`);
  return at === -1 ? fallback : (process.argv[at + 1] ?? fallback);
}

const seconds = Number(option("seconds", "12"));
const [width, height] = option("size", "1920x1080").split("x").map(Number);
const base = option("url", "http://localhost:1420");
const beat = option("beat", "");
const blur = process.argv.includes("--blur");
const out = resolve(option("out", join(app, "..", ".dev", "reel", "reel.mp4")));
const url = `${base.replace(/\/$/, "")}/?reel${beat ? `&beat=${beat}` : ""}`;

const BROWSERS = {
  win32: [
    "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
    "C:/Program Files/Google/Chrome/Application/chrome.exe",
  ],
  darwin: [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  ],
  linux: ["/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"],
};
const browserPath =
  process.env.REEL_BROWSER ?? (BROWSERS[process.platform] ?? []).find((path) => existsSync(path));
if (!browserPath) {
  throw new Error("no Chromium browser found: set REEL_BROWSER");
}

const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

/** Waits until `read` returns something, for up to 15 s. */
async function until(read) {
  for (let tries = 0; tries < 150; tries++) {
    try {
      const value = await read();
      if (value) return value;
    } catch {}
    await sleep(100);
  }
  throw new Error("the browser did not start");
}

/**
 * A picture of the page as it is. Now and then the browser never answers
 * one; a fresh paint (the clock's corner pixel, stepped by nothing) and
 * another try get it going again.
 */
async function picture(cdp) {
  for (let tries = 1; ; tries++) {
    try {
      const shot = { format: "jpeg", quality: 95 };
      return (await cdp.send("Page.captureScreenshot", shot, 5_000)).data;
    } catch (error) {
      if (tries === 5) throw error;
      await cdp.send("Runtime.evaluate", { expression: "__reelStep(0)", awaitPromise: true });
    }
  }
}

/** A small DevTools protocol client over the page's WebSocket. */
function devtools(socket) {
  let next = 0;
  const waiting = new Map();
  const listeners = new Map();
  socket.addEventListener("message", ({ data }) => {
    const message = JSON.parse(data);
    if (message.id !== undefined) {
      const call = waiting.get(message.id);
      waiting.delete(message.id);
      if (message.error) call?.fail(new Error(message.error.message));
      else call?.done(message.result);
    } else {
      for (const listen of listeners.get(message.method) ?? []) listen(message.params);
    }
  });
  return {
    send(method, params = {}, wait = 60_000) {
      const id = ++next;
      socket.send(JSON.stringify({ id, method, params }));
      // A call the browser never answers fails instead of hanging the run.
      return new Promise((done, fail) => {
        const timer = setTimeout(() => fail(new Error(`${method} got no answer`)), wait);
        waiting.set(id, {
          done: (result) => {
            clearTimeout(timer);
            done(result);
          },
          fail: (error) => {
            clearTimeout(timer);
            fail(error);
          },
        });
      });
    },
    on(method, listen) {
      listeners.set(method, [...(listeners.get(method) ?? []), listen]);
    },
    once(method) {
      return new Promise((done) => this.on(method, done));
    },
  };
}

const work = mkdtempSync(join(tmpdir(), "botloft-reel-"));
const profile = join(work, "profile");
const frames = join(work, "frames");
mkdirSync(frames, { recursive: true });

const browser = spawn(
  browserPath,
  [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    `--window-size=${width},${height}`,
    "--force-device-scale-factor=1",
    // Windows takes the headless window for one covered by others and stops
    // painting it after a moment.
    "--disable-features=CalculateNativeWinOcclusion",
    "--disable-backgrounding-occluded-windows",
    "--disable-renderer-backgrounding",
    "--hide-scrollbars",
    "--mute-audio",
    "--no-first-run",
    "about:blank",
  ],
  { stdio: "ignore" },
);

try {
  const port = await until(
    () => readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0],
  );
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  const page = targets.find((target) => target.type === "page");
  const socket = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((done) => socket.addEventListener("open", done));
  const cdp = devtools(socket);

  await cdp.send("Page.enable");
  // A page Windows thinks is out of sight paints about once a second.
  await cdp.send("Page.bringToFront");
  await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: true });
  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width,
    height,
    deviceScaleFactor: 1,
    mobile: false,
  });
  // The page's clock stands still from the start (reelClock.js), on the
  // reel's own dark rather than the app's light background.
  const clock = readFileSync(join(app, "scripts", "reelClock.js"), "utf8");
  const dark =
    "const sheet = new CSSStyleSheet();" +
    "sheet.replaceSync('html, body { background: #050505 !important; }');" +
    "document.adoptedStyleSheets = [sheet];";
  await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: `${clock}\n${dark}` });
  const loaded = cdp.once("Page.loadEventFired");
  await cdp.send("Page.navigate", { url });
  await loaded;
  await until(async () => {
    const { result } = await cdp.send("Runtime.evaluate", {
      expression: "document.querySelector('.reel-place') !== null",
    });
    return result.value;
  });

  const total = Math.round(seconds * 60);
  for (let index = 0; index < total; index++) {
    if (index > 0) {
      await cdp.send("Runtime.evaluate", {
        expression: `__reelStep(${1000 / 60})`,
        awaitPromise: true,
      });
    }
    const data = await picture(cdp);
    const name = `frame_${String(index + 1).padStart(5, "0")}.jpg`;
    writeFileSync(join(frames, name), Buffer.from(data, "base64"));
    if ((index + 1) % 60 === 0) {
      process.stdout.write(`${(index + 1) / 60} s of ${seconds}\r`);
    }
  }
  await cdp.send("Browser.close").catch(() => {});

  mkdirSync(dirname(out), { recursive: true });
  const filters = [...(blur ? ["tmix=frames=3:weights=1 2 1"] : []), "format=yuv420p"];
  execFileSync(
    "ffmpeg",
    [
      "-y",
      "-v",
      "error",
      ...["-framerate", "60", "-i", join(frames, "frame_%05d.jpg")],
      ...["-vf", filters.join(","), "-c:v", "libx264", "-crf", "14", "-preset", "slow"],
      ...["-movflags", "+faststart", out],
    ],
    { stdio: "inherit" },
  );
  console.log(out);
} finally {
  // The browser runs as a tree of processes; on Windows killing the first
  // one leaves the rest holding the profile folder.
  if (process.platform === "win32") {
    try {
      execFileSync("taskkill", ["/pid", String(browser.pid), "/t", "/f"], { stdio: "ignore" });
    } catch {}
  }
  browser.kill();
  await sleep(300);
  rmSync(work, { recursive: true, force: true });
}
