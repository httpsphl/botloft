// A tool's square (spec 34.7), as on claude.ai: the tool's official icon
// on white for the tools the app knows, a plug on gray for the rest.

import { Plug } from "lucide-react";
import { knownTool, toolIcon } from "./toolCatalog";

export function ToolTile({ name, size = 40 }: { name: string; size?: number }) {
  const known = knownTool(name);
  const icon = known && toolIcon(known.key);
  const box = { width: size, height: size };
  if (!icon) {
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
  const inner = Math.round(size * 0.74);
  return (
    <span
      aria-hidden
      className="grid shrink-0 place-items-center rounded-xl bg-white ring-1 ring-black/10"
      style={box}
    >
      <img
        src={icon}
        alt=""
        draggable={false}
        className="object-contain"
        style={{ width: inner, height: inner, borderRadius: Math.round(inner * 0.22) }}
      />
    </span>
  );
}
