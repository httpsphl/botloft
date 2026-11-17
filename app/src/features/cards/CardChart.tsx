// A bar or line chart in an agent's card (spec 8.11). One y axis from
// zero, thin marks in the series colors of cards.css, a legend from two
// series on, a hover tip per label and the same numbers as a table, since
// some series colors sit under 3:1 on the light card.

import { useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { CardBlock } from "../../lib/protocol.gen";
import { CardTable } from "./CardTable";
import { labelEvery, scaleOf } from "./chartScale";

type Chart = Extract<CardBlock, { type: "chart" }>;

const HEIGHT = 200;
const PAD = { top: 8, right: 8, bottom: 24, left: 44 };
const number = new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 });

const color = (at: number) => `var(--series-${(at % 5) + 1})`;

export function CardChart({ chart }: { chart: Chart }) {
  const t = useT().agentCards.chart;
  const [asTable, setAsTable] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(560);
  const [hover, setHover] = useState<number | null>(null);

  useEffect(() => {
    const node = box.current;
    if (!node || typeof ResizeObserver === "undefined") {
      return;
    }
    const watch = new ResizeObserver(([entry]) => {
      if (entry) {
        setWidth(Math.max(240, Math.floor(entry.contentRect.width)));
      }
    });
    watch.observe(node);
    return () => watch.disconnect();
  }, []);

  const unit = chart.unit ? ` ${chart.unit}` : "";
  const show = (value: number | null) => (value === null ? "—" : `${number.format(value)}${unit}`);
  const names = chart.series.map((one) => one.name).join(", ");
  const toggle = (
    <button
      type="button"
      onClick={() => setAsTable((now) => !now)}
      className="self-start text-muted text-xs underline-offset-2 hover:text-ink hover:underline"
    >
      {asTable ? t.asChart : t.asTable}
    </button>
  );
  if (asTable) {
    return (
      <div className="flex flex-col gap-1.5">
        <CardTable
          columns={[chart.unit ?? t.label0, ...chart.series.map((one) => one.name)]}
          rows={chart.labels.map((label, at) => [
            label,
            ...chart.series.map((one) => show(one.values[at] ?? null)),
          ])}
        />
        {toggle}
      </div>
    );
  }

  const scale = scaleOf(chart.series.flatMap((one) => one.values));
  const plotW = width - PAD.left - PAD.right;
  const plotH = HEIGHT - PAD.top - PAD.bottom;
  const y = (value: number) =>
    PAD.top + plotH - ((value - scale.min) / (scale.max - scale.min)) * plotH;
  const band = plotW / chart.labels.length;
  const x = (at: number) => PAD.left + band * at + band / 2;
  const every = labelEvery(chart.labels.length, plotW);
  const zero = y(0);
  const tipAt = hover === null ? null : Math.min(Math.max(x(hover), 90), width - 90);

  return (
    <div className="flex flex-col gap-1.5">
      {chart.series.length > 1 && (
        <ul className="flex flex-wrap gap-x-4 gap-y-1 text-ink-soft text-xs">
          {chart.series.map((one, at) => (
            <li key={one.name} className="flex items-center gap-1.5">
              <span
                aria-hidden
                className="h-2.5 w-2.5 rounded-sm"
                style={{ background: color(at) }}
              />
              {one.name}
            </li>
          ))}
        </ul>
      )}
      <div ref={box} className="relative w-full" onPointerLeave={() => setHover(null)}>
        <svg
          role="img"
          aria-label={t.label(chart.kind === "bar" ? t.bar : t.line, names)}
          width={width}
          height={HEIGHT}
          className="block overflow-visible"
        >
          {scale.ticks.map((tick) => (
            <g key={tick}>
              <line
                x1={PAD.left}
                x2={width - PAD.right}
                y1={y(tick)}
                y2={y(tick)}
                className="card-chart-grid"
              />
              <text
                x={PAD.left - 6}
                y={y(tick)}
                dy="0.32em"
                textAnchor="end"
                className="card-chart-tick"
              >
                {number.format(tick)}
              </text>
            </g>
          ))}
          {hover !== null && chart.kind === "bar" && (
            <rect
              x={PAD.left + band * hover}
              y={PAD.top}
              width={band}
              height={plotH}
              className="card-chart-hover"
            />
          )}
          {chart.kind === "bar" ? (
            <Bars chart={chart} band={band} x={x} y={y} zero={zero} />
          ) : (
            <Lines chart={chart} x={x} y={y} />
          )}
          {hover !== null && chart.kind === "line" && (
            <line
              x1={x(hover)}
              x2={x(hover)}
              y1={PAD.top}
              y2={PAD.top + plotH}
              className="card-chart-cross"
            />
          )}
          {chart.labels.map((label, at) =>
            at % every === 0 ? (
              <text
                key={label}
                x={x(at)}
                y={HEIGHT - 6}
                textAnchor="middle"
                className="card-chart-tick"
              >
                {label}
              </text>
            ) : null,
          )}
          {chart.labels.map((label, at) => (
            <rect
              key={`hit-${label}`}
              x={PAD.left + band * at}
              y={PAD.top}
              width={band}
              height={plotH}
              fill="transparent"
              onPointerEnter={() => setHover(at)}
            />
          ))}
        </svg>
        {hover !== null && tipAt !== null && (
          <div
            role="tooltip"
            className="pointer-events-none absolute top-0 z-10 min-w-36 -translate-x-1/2 rounded-lg border border-line bg-panel px-2.5 py-1.5 text-xs shadow-lg"
            style={{ left: tipAt }}
          >
            <p className="mb-1 font-semibold">{chart.labels[hover]}</p>
            {chart.series.map((one, at) => (
              <p key={one.name} className="flex items-center gap-1.5 text-ink-soft">
                <span
                  aria-hidden
                  className="h-2 w-2 rounded-sm"
                  style={{ background: color(at) }}
                />
                <span className="flex-1">{one.name}</span>
                <span className="text-ink tabular-nums">{show(one.values[hover] ?? null)}</span>
              </p>
            ))}
          </div>
        )}
      </div>
      {toggle}
    </div>
  );
}

function Bars({
  chart,
  band,
  x,
  y,
  zero,
}: {
  chart: Chart;
  band: number;
  x: (at: number) => number;
  y: (value: number) => number;
  zero: number;
}) {
  const count = chart.series.length;
  const group = Math.min(band * 0.72, 28 * count);
  const bar = Math.max(2, (group - 2 * (count - 1)) / count);
  return (
    <g>
      {chart.labels.map((label, at) =>
        chart.series.map((one, s) => {
          const value = one.values[at];
          if (value === null || value === undefined) {
            return null;
          }
          const left = x(at) - group / 2 + s * (bar + 2);
          const top = Math.min(y(value), zero);
          const height = Math.max(1, Math.abs(zero - y(value)));
          const r = Math.min(4, bar / 2, height);
          // Rounded at the data end, square on the baseline.
          const up = value >= 0;
          const d = up
            ? `M${left},${zero}V${top + r}Q${left},${top} ${left + r},${top}H${left + bar - r}Q${left + bar},${top} ${left + bar},${top + r}V${zero}Z`
            : `M${left},${zero}V${top + height - r}Q${left},${top + height} ${left + r},${top + height}H${left + bar - r}Q${left + bar},${top + height} ${left + bar},${top + height - r}V${zero}Z`;
          return <path key={`${label}-${one.name}`} d={d} fill={color(s)} />;
        }),
      )}
    </g>
  );
}

function Lines({
  chart,
  x,
  y,
}: {
  chart: Chart;
  x: (at: number) => number;
  y: (value: number) => number;
}) {
  return (
    <g>
      {chart.series.map((one, s) => {
        // A gap where a value is missing.
        let d = "";
        let open = false;
        one.values.forEach((value, at) => {
          if (value === null) {
            open = false;
            return;
          }
          d += `${open ? "L" : "M"}${x(at)},${y(value)}`;
          open = true;
        });
        return (
          <g key={one.name}>
            <path d={d} fill="none" stroke={color(s)} strokeWidth={2} strokeLinejoin="round" />
            {one.values.map((value, at) =>
              value === null ? null : (
                <circle
                  // biome-ignore lint/suspicious/noArrayIndexKey: one point per label
                  key={at}
                  cx={x(at)}
                  cy={y(value)}
                  r={4}
                  fill={color(s)}
                  className="card-chart-point"
                />
              ),
            )}
          </g>
        );
      })}
    </g>
  );
}
