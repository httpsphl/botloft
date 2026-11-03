// A site's square (spec 35.4): the official icon when the site is one of
// the tools the app knows (34.7), its initial otherwise.

import { knownTool, toolIcon } from "../connections/toolCatalog";

/** "reddit.com" names the tool "Reddit"; "app.asana.com", "Asana". */
function asTool(site: string) {
  const parts = site.split(".");
  const name = parts.length > 1 ? parts[parts.length - 2] : site;
  const tool = name ? knownTool(name) : undefined;
  return tool ? toolIcon(tool.key) : undefined;
}

export function SiteTile({ site, size = 40 }: { site: string; size?: number }) {
  const icon = asTool(site);
  const box = { width: size, height: size };
  if (icon) {
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
  return (
    <span
      aria-hidden
      className="grid shrink-0 place-items-center rounded-xl bg-sunken font-semibold text-ink-soft"
      style={{ ...box, fontSize: Math.round(size * 0.42) }}
    >
      {site.charAt(0).toUpperCase()}
    </span>
  );
}
