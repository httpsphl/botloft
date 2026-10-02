// A free color picker for a bot's avatar: a saturation/brightness square
// over the chosen hue, a hue slider, and the color as hex and as red, green
// and blue. Everything edits the same color, and `onChange` gets it as
// `#RRGGBB`. The square answers to the arrow keys as well as the pointer.

import { type KeyboardEvent, type PointerEvent, useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import { type Hsv, hsvToRgb, parseHex, type Rgb, rgbToHsv, toHexColor } from "./colorSpace";

const HUES =
  "linear-gradient(to right, #f00 0%, #ff0 17%, #0f0 33%, #0ff 50%, #00f 67%, #f0f 83%, #f00 100%)";

const unit = (value: number) => Math.max(0, Math.min(1, value));

export function ColorPicker({ value, onChange }: { value: string; onChange(hex: string): void }) {
  const t = useT().bots.dialog.picker;
  const [hsv, setHsv] = useState<Hsv>(() => rgbToHsv(parseHex(value) ?? [255, 122, 89]));
  const [hexDraft, setHexDraft] = useState(value);
  const area = useRef<HTMLDivElement>(null);
  const rgb = hsvToRgb(hsv);
  const hex = toHexColor(rgb);

  // A color set from outside (a palette swatch) moves the picker too, but
  // the picker's own changes keep the hue it holds.
  useEffect(() => {
    const outside = parseHex(value);
    if (outside) {
      setHsv((held) =>
        toHexColor(hsvToRgb(held)) === toHexColor(outside) ? held : rgbToHsv(outside),
      );
    }
    setHexDraft(value.toUpperCase());
  }, [value]);

  const pick = (next: Hsv) => {
    setHsv(next);
    const nextHex = toHexColor(hsvToRgb(next));
    setHexDraft(nextHex);
    onChange(nextHex);
  };

  const pickRgb = (next: Rgb) => pick(rgbToHsv(next));

  const fromPointer = (event: PointerEvent<HTMLDivElement>) => {
    const box = area.current?.getBoundingClientRect();
    if (!box || box.width === 0 || box.height === 0) {
      return;
    }
    pick({
      h: hsv.h,
      s: unit((event.clientX - box.left) / box.width),
      v: 1 - unit((event.clientY - box.top) / box.height),
    });
  };

  const onAreaKey = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? 0.1 : 0.02;
    const moves: Record<string, Partial<Hsv>> = {
      ArrowLeft: { s: unit(hsv.s - step) },
      ArrowRight: { s: unit(hsv.s + step) },
      ArrowUp: { v: unit(hsv.v + step) },
      ArrowDown: { v: unit(hsv.v - step) },
    };
    const move = moves[event.key];
    if (move) {
      event.preventDefault();
      pick({ ...hsv, ...move });
    }
  };

  const percent = (n: number) => `${Math.round(n * 100)}%`;

  return (
    <div className="flex flex-col gap-2.5 rounded-xl border border-line bg-sunken p-3">
      <div
        ref={area}
        role="slider"
        tabIndex={0}
        aria-label={t.area}
        aria-valuetext={t.areaValue(percent(hsv.s), percent(hsv.v))}
        aria-valuenow={Math.round(hsv.s * 100)}
        onKeyDown={onAreaKey}
        onPointerDown={(event) => {
          event.currentTarget.setPointerCapture(event.pointerId);
          fromPointer(event);
        }}
        onPointerMove={(event) => {
          if (event.currentTarget.hasPointerCapture(event.pointerId)) {
            fromPointer(event);
          }
        }}
        className="relative h-36 cursor-crosshair touch-none rounded-lg"
        style={{
          backgroundColor: toHexColor(hsvToRgb({ h: hsv.h, s: 1, v: 1 })),
          backgroundImage:
            "linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, transparent)",
        }}
      >
        <span
          aria-hidden
          className="pointer-events-none absolute size-3.5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgb(0_0_0/0.5)]"
          style={{ left: percent(hsv.s), top: percent(1 - hsv.v), backgroundColor: hex }}
        />
      </div>
      <input
        type="range"
        min={0}
        max={359}
        value={Math.round(hsv.h)}
        aria-label={t.hue}
        onChange={(event) => pick({ ...hsv, h: Number(event.target.value) })}
        className="color-hue h-3 w-full cursor-pointer appearance-none rounded-full"
        style={{ backgroundImage: HUES }}
      />
      <div className="flex items-end gap-2">
        <label className="flex min-w-0 flex-[1.6] flex-col gap-1 text-muted text-xs">
          {t.hex}
          <input
            value={hexDraft}
            maxLength={7}
            spellCheck={false}
            onChange={(event) => {
              setHexDraft(event.target.value);
              const parsed = parseHex(event.target.value);
              if (parsed) {
                pickRgb(parsed);
              }
            }}
            onBlur={() => setHexDraft(hex)}
            className="h-8 w-full rounded-lg border border-line-strong bg-canvas px-2 font-mono text-ink text-sm uppercase outline-none focus:border-accent"
          />
        </label>
        {(["r", "g", "b"] as const).map((channel, index) => (
          <label key={channel} className="flex min-w-0 flex-1 flex-col gap-1 text-muted text-xs">
            {t.channels[channel]}
            <input
              type="number"
              min={0}
              max={255}
              value={rgb[index]}
              onChange={(event) => {
                const next: Rgb = [...rgb];
                next[index] = Math.max(0, Math.min(255, Number(event.target.value) || 0));
                pickRgb(next);
              }}
              className="h-8 w-full rounded-lg border border-line-strong bg-canvas px-2 font-mono text-ink text-sm outline-none focus:border-accent"
            />
          </label>
        ))}
      </div>
    </div>
  );
}
