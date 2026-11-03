// A tool's square (spec 34.7), as on claude.ai: the brand's mark in its
// color on white, or dark on its color when that color is too light to
// read on white; the initial on its color for a brand with no mark; a plug
// on gray for a tool the app does not know.

import { Plug } from "lucide-react";
import { contrastOnWhite, knownTool } from "./toolCatalog";

const INK = "#111111";

export function ToolTile({ name, size = 40 }: { name: string; size?: number }) {
  const known = knownTool(name);
  const box = { width: size, height: size };
  if (!known) {
    return (
      <span
        aria-hidden
        className="grid shrink-0 place-items-center rounded-xl bg-sunken text-ink-soft"
        style={box}
      >
        <Plug size={Math.round(size * 0.5)} strokeWidth={2} />
      </span>
    );
  }
  const pale = contrastOnWhite(known.color) < 2.2;
  if (!known.mark) {
    return (
      <span
        aria-hidden
        className="grid shrink-0 place-items-center rounded-xl font-bold"
        style={{
          ...box,
          background: known.color,
          color: pale ? INK : "#ffffff",
          fontSize: Math.round(size * 0.46),
        }}
      >
        {known.name.charAt(0).toUpperCase()}
      </span>
    );
  }
  const mark = Math.round(size * 0.56);
  return (
    <span
      aria-hidden
      className="grid shrink-0 place-items-center rounded-xl ring-1 ring-black/10"
      style={{ ...box, background: pale ? known.color : "#ffffff" }}
    >
      <svg
        aria-hidden
        viewBox="0 0 24 24"
        width={mark}
        height={mark}
        fill={pale ? INK : known.color}
      >
        <path d={known.mark.path} />
      </svg>
    </span>
  );
}
