// A tool's square (spec 34.7): its icon, white on its color, for the tools
// the app knows; a plug on gray for the rest.

import { Plug } from "lucide-react";
import { knownTool } from "./toolCatalog";

export function ToolTile({ name, size = 40 }: { name: string; size?: number }) {
  const known = knownTool(name);
  const Icon = known?.icon ?? Plug;
  return (
    <span
      aria-hidden
      className={`grid shrink-0 place-items-center rounded-xl ${known ? "text-white" : "bg-sunken text-ink-soft"}`}
      style={{ width: size, height: size, background: known?.color }}
    >
      <Icon size={Math.round(size * 0.5)} strokeWidth={2} />
    </span>
  );
}
