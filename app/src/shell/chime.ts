// A short, soft sound for a notification while Botloft is in front (spec
// 15.1): two notes made on the spot, so no sound file ships.

type AudioWindow = Window & { AudioContext?: typeof AudioContext };

let context: AudioContext | null = null;

/** Two notes, rising when a bot needs the owner and falling when one is done. */
export function chime(kind: "needs" | "done"): void {
  const Audio = (window as AudioWindow).AudioContext;
  if (!Audio) {
    return;
  }
  try {
    context ??= new Audio();
    const start = context.currentTime + 0.01;
    const notes = kind === "needs" ? [660, 880] : [784, 587];
    notes.forEach((frequency, index) => {
      const at = start + index * 0.12;
      const tone = context?.createOscillator();
      const volume = context?.createGain();
      if (!context || !tone || !volume) {
        return;
      }
      tone.type = "sine";
      tone.frequency.value = frequency;
      volume.gain.setValueAtTime(0.0001, at);
      volume.gain.exponentialRampToValueAtTime(0.12, at + 0.015);
      volume.gain.exponentialRampToValueAtTime(0.0001, at + 0.22);
      tone.connect(volume).connect(context.destination);
      tone.start(at);
      tone.stop(at + 0.25);
    });
  } catch {
    // No sound device: the notification says it anyway.
  }
}
