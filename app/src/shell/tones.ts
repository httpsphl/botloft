// The app's sounds (spec 15.1): a few soft notes made on the spot, so no
// sound file ships. All of them share one timbre and stay quieter than the
// chime of a bot that needs the owner.

export type Sound = "needs" | "done" | "sent" | "reply" | "file" | "spark";

interface Note {
  /** Hz at the start, and where it slides to, if anywhere. */
  from: number;
  to?: number;
  /** Seconds after the sound starts. */
  at: number;
  length: number;
  gain: number;
}

const note = (from: number, at: number, length: number, gain: number, to?: number): Note => ({
  from,
  at,
  length,
  gain,
  ...(to ? { to } : {}),
});

const SOUNDS: Record<Sound, Note[]> = {
  // Two notes rising: a bot needs the owner.
  needs: [note(660, 0, 0.22, 0.12), note(880, 0.12, 0.22, 0.12)],
  // Two notes falling: a bot finished.
  done: [note(784, 0, 0.22, 0.12), note(587, 0.12, 0.22, 0.12)],
  // A short, high tick: the message left.
  sent: [note(1046, 0, 0.06, 0.04)],
  // A soft pop that settles: a reply came in the open chat.
  reply: [note(880, 0, 0.12, 0.06, 660)],
  // Two bright notes close together: a file arrived.
  file: [note(784, 0, 0.1, 0.06), note(1175, 0.07, 0.14, 0.06)],
  // A quick rising spark, like the flame: a bot or a crew was born.
  spark: [note(784, 0, 0.1, 0.06), note(988, 0.06, 0.1, 0.06), note(1319, 0.12, 0.2, 0.07, 1397)],
};

type AudioWindow = Window & { AudioContext?: typeof AudioContext };

let context: AudioContext | null = null;

export function tones(sound: Sound): void {
  const Audio = (window as AudioWindow).AudioContext;
  if (!Audio) {
    return;
  }
  try {
    context ??= new Audio();
    const start = context.currentTime + 0.01;
    for (const { from, to, at, length, gain } of SOUNDS[sound]) {
      const begin = start + at;
      const tone = context.createOscillator();
      const volume = context.createGain();
      tone.type = "sine";
      tone.frequency.setValueAtTime(from, begin);
      if (to) {
        tone.frequency.exponentialRampToValueAtTime(to, begin + length);
      }
      volume.gain.setValueAtTime(0.0001, begin);
      volume.gain.exponentialRampToValueAtTime(gain, begin + 0.012);
      volume.gain.exponentialRampToValueAtTime(0.0001, begin + length);
      tone.connect(volume).connect(context.destination);
      tone.start(begin);
      tone.stop(begin + length + 0.03);
    }
  } catch {
    // No sound device: nothing else depends on it.
  }
}
