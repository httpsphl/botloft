// The numbers behind a card's chart (spec 8.11): one y scale from zero
// through the values, with round ticks. Pure, so it is tested alone.

/** A round step near `raw`: 1, 2, 2.5 or 5 times a power of ten. */
export function niceStep(raw: number): number {
  if (!Number.isFinite(raw) || raw <= 0) {
    return 1;
  }
  const power = 10 ** Math.floor(Math.log10(raw));
  const unit = raw / power;
  const nice = unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10;
  return nice * power;
}

export interface Scale {
  min: number;
  max: number;
  ticks: number[];
}

/** The y scale for `values`: zero always in it, about four round steps. */
export function scaleOf(values: (number | null)[]): Scale {
  const known = values.filter((value): value is number => value !== null);
  const low = Math.min(0, ...known);
  const high = Math.max(0, ...known);
  if (low === high) {
    return { min: 0, max: 1, ticks: [0, 1] };
  }
  const step = niceStep((high - low) / 4);
  const min = Math.floor(low / step) * step;
  const max = Math.ceil(high / step) * step;
  const ticks: number[] = [];
  for (let tick = min; tick <= max + step / 2; tick += step) {
    // Floats add up crumbs: 0.30000000000000004 reads as 0.3.
    ticks.push(Number(tick.toPrecision(12)));
  }
  return { min, max, ticks };
}

/** Which labels to write under the chart so they do not run together. */
export function labelEvery(count: number, width: number, room = 56): number {
  return Math.max(1, Math.ceil((count * room) / Math.max(width, 1)));
}
