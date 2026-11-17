// A sound of the bot's, played right in the chat (spec 15.1): one button,
// a bar to jump around and the time. The browser's own controls would look
// like no other part of the app.

import { Pause, Play } from "lucide-react";
import { type CSSProperties, useRef, useState } from "react";
import { useT } from "../../i18n";
import { clock, startedPlaying } from "./media";

export function AudioPlayer({ src, name, onFail }: { src: string; name: string; onFail(): void }) {
  const t = useT().files.media;
  const audio = useRef<HTMLAudioElement>(null);
  const [playing, setPlaying] = useState(false);
  const [at, setAt] = useState(0);
  // A recording still being written may not know its length: 0 then.
  const [length, setLength] = useState(0);
  const known = (seconds: number) => (Number.isFinite(seconds) ? seconds : 0);
  const toggle = () => {
    const element = audio.current;
    if (!element) {
      return;
    }
    if (element.paused) {
      void element.play().catch(() => setPlaying(false));
    } else {
      element.pause();
    }
  };
  const played = length > 0 ? Math.min(100, (at / length) * 100) : 0;
  return (
    <div className="flex items-center gap-2.5 px-3 py-2.5">
      {/* biome-ignore lint/a11y/useMediaCaption: the bot's own recordings have no captions */}
      <audio
        ref={audio}
        src={src}
        preload="metadata"
        onPlay={(event) => {
          startedPlaying(event.currentTarget);
          setPlaying(true);
        }}
        onPause={() => setPlaying(false)}
        onEnded={() => setPlaying(false)}
        onTimeUpdate={(event) => setAt(event.currentTarget.currentTime)}
        onLoadedMetadata={(event) => setLength(known(event.currentTarget.duration))}
        onDurationChange={(event) => setLength(known(event.currentTarget.duration))}
        onError={onFail}
      />
      <button
        type="button"
        aria-label={playing ? t.pause(name) : t.play(name)}
        title={playing ? t.pause(name) : t.play(name)}
        onClick={toggle}
        className="grid size-9 shrink-0 place-items-center rounded-full bg-ink text-canvas transition-colors hover:bg-ink-soft"
      >
        {playing ? (
          <Pause aria-hidden size={16} fill="currentColor" />
        ) : (
          <Play aria-hidden size={16} fill="currentColor" className="translate-x-px" />
        )}
      </button>
      <input
        type="range"
        min={0}
        max={length || 1}
        step="any"
        value={length ? at : 0}
        disabled={!length}
        aria-label={t.position(name)}
        aria-valuetext={`${clock(at)} / ${clock(length)}`}
        onChange={(event) => {
          const element = audio.current;
          const seconds = Number(event.currentTarget.value);
          if (element) {
            element.currentTime = seconds;
          }
          setAt(seconds);
        }}
        className="media-seek min-w-0 flex-1"
        style={{ "--played": `${played}%` } as CSSProperties}
      />
      <span className="shrink-0 text-muted text-xs tabular-nums">
        {length ? `${clock(at)} / ${clock(length)}` : clock(at)}
      </span>
    </div>
  );
}
