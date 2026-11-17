// A turn that makes media, for the preview (spec 15.1): Scout shares a
// voice-over, and the teaser and cover a command made show under the turn
// as "Also made". All of it plays for real: the sound is written here, the
// cover drawn on a canvas and the teaser recorded from one, which takes a
// few seconds, so the turn arrives live once the recording is done.

import { encodeBytes } from "../lib/base64";
import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const WIDTH = 640;
const HEIGHT = 360;

/** A short rising tune as a 16-bit mono WAV. */
function tune(): Uint8Array<ArrayBuffer> {
  const rate = 22_050;
  const notes = [523.25, 659.25, 783.99, 1046.5];
  const each = 0.5;
  const samples = Math.floor(rate * each * notes.length);
  const view = new DataView(new ArrayBuffer(44 + samples * 2));
  const text = (offset: number, value: string) => {
    for (let index = 0; index < value.length; index++) {
      view.setUint8(offset + index, value.charCodeAt(index));
    }
  };
  text(0, "RIFF");
  view.setUint32(4, 36 + samples * 2, true);
  text(8, "WAVEfmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, rate, true);
  view.setUint32(28, rate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  text(36, "data");
  view.setUint32(40, samples * 2, true);
  for (let index = 0; index < samples; index++) {
    const time = index / rate;
    const note = notes[Math.min(notes.length - 1, Math.floor(time / each))] ?? 440;
    const local = time % each;
    const envelope = Math.min(1, local * 40) * Math.exp(-local * 4);
    const sample = Math.sin(2 * Math.PI * note * time) * envelope * 0.35;
    view.setInt16(44 + index * 2, Math.round(sample * 32_767), true);
  }
  return new Uint8Array(view.buffer);
}

/** One frame of the cover and the teaser; `progress` runs from 0 to 1. */
function paint(context: CanvasRenderingContext2D, progress: number): void {
  const gradient = context.createLinearGradient(0, 0, WIDTH, HEIGHT);
  gradient.addColorStop(0, "#fca379");
  gradient.addColorStop(1, "#d9481f");
  context.fillStyle = gradient;
  context.fillRect(0, 0, WIDTH, HEIGHT);
  context.fillStyle = "rgba(255, 255, 255, 0.85)";
  context.beginPath();
  context.arc(90 + progress * (WIDTH - 180), HEIGHT * 0.72, 28, 0, Math.PI * 2);
  context.fill();
  context.fillStyle = "#1f1a17";
  context.font = "600 52px 'IBM Plex Sans', sans-serif";
  context.fillText("Week 39", 60, 140);
  context.font = "400 24px 'IBM Plex Sans', sans-serif";
  context.fillText("What the papers say, in a minute", 60, 182);
}

function canvas(): { element: HTMLCanvasElement; context: CanvasRenderingContext2D } | null {
  const element = document.createElement("canvas");
  element.width = WIDTH;
  element.height = HEIGHT;
  const context = element.getContext("2d");
  return context ? { element, context } : null;
}

/** The cover as a PNG, in base64. */
function cover(): string | null {
  const drawn = canvas();
  if (!drawn) {
    return null;
  }
  paint(drawn.context, 0.5);
  const url = drawn.element.toDataURL("image/png");
  return url.slice(url.indexOf(",") + 1);
}

/** A few seconds recorded from a canvas, as a WebM the browser plays. */
async function teaser(seconds: number): Promise<Blob | null> {
  const drawn = canvas();
  if (!drawn || typeof MediaRecorder === "undefined") {
    return null;
  }
  const recorder = new MediaRecorder(drawn.element.captureStream(30), { mimeType: "video/webm" });
  const chunks: Blob[] = [];
  recorder.ondataavailable = (event) => chunks.push(event.data);
  const stopped = new Promise<void>((resolve) => {
    recorder.onstop = () => resolve();
  });
  recorder.start();
  const start = performance.now();
  // Timers, not animation frames: those stop while the page is hidden.
  await new Promise<void>((resolve) => {
    const timer = setInterval(() => {
      const progress = (performance.now() - start) / (seconds * 1000);
      paint(drawn.context, Math.min(1, progress));
      if (progress >= 1) {
        clearInterval(timer);
        resolve();
      }
    }, 1000 / 30);
  });
  recorder.stop();
  await stopped;
  return new Blob(chunks, { type: "video/webm" });
}

export function seedMedia(fake: FakeBotloft, scout: BotId): void {
  const { chat, files } = fake;
  void teaser(3).then((video) => {
    fake.now = Date.now();
    fake.setBotState(scout, "busy");
    void fake.call("messages.send", {
      botId: scout,
      body: "Make a short teaser of the week, with a voice-over and a cover.",
    });
    const command = chat.tool(scout, "Bash", {
      summary: "ffmpeg -loop 1 -i cover.png -i voice-over.wav -t 3 teaser.webm",
    });
    chat.finish(command, "done");
    const voice = tune();
    const sound = files.add(scout, "voice-over.wav", {
      data: encodeBytes(voice),
      folder: "teaser",
    });
    files.play(sound.path, URL.createObjectURL(new Blob([voice], { type: "audio/wav" })));
    const picture = cover();
    if (picture) {
      files.add(scout, "cover.png", { data: picture, folder: "teaser" });
    }
    if (video) {
      const clip = files.add(scout, "teaser.webm", {
        text: "webm",
        size: video.size,
        folder: "teaser",
      });
      files.play(clip.path, URL.createObjectURL(video));
    }
    const shared = chat.tool(scout, "mcp__botloft__share_file", {
      summary: "voice-over.wav",
      input: JSON.stringify({ files: [sound.path] }),
    });
    chat.finish(shared, JSON.stringify({ shown: [sound] }));
    chat.reply(scout, "Here's the voice-over. The teaser and its cover are in the same folder.");
    chat.turn(scout);
    fake.setBotState(scout, "idle");
  });
}
