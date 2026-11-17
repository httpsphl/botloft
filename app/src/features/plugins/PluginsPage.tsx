// Settings > Plugins (spec 41.5): the plugins on in the owner's own Claude
// Code, each a card with what it does, where it comes from, how many
// skills it brings and how many agents use it; the card opens what it
// brings and who may use it. Every plugin can go to every agent at once,
// and a search finds one among many.

import { Puzzle, Search, TriangleAlert } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { plain } from "../account/settingsIndex";
import { AllForEveryone } from "../skills/AllForEveryone";
import { PluginDialog } from "./PluginDialog";
import { runsByItself, warningsOf } from "./PluginWarnings";
import { usePlugins } from "./usePlugins";

/** With more plugins than this, the page has a search. */
const SEARCH_FROM = 6;

export function PluginsPage() {
  const words = useT().plugins;
  const { plugins, everyone, share, forEveryone } = usePlugins(
    words.shareFailed,
    words.everyone.failed,
  );
  const [open, setOpen] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const opened = plugins?.find((plugin) => plugin.name === open);
  const wanted = plain(query);
  const shown =
    plugins?.filter(
      (plugin) =>
        wanted === "" ||
        plain(plugin.name).includes(wanted) ||
        plain(plugin.description).includes(wanted),
    ) ?? [];
  return (
    <div className="flex flex-col gap-4">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      {plugins !== null && plugins.length === 0 && (
        <p className="text-muted text-sm leading-relaxed">{words.none}</p>
      )}
      {plugins !== null && plugins.length > 0 && (
        <AllForEveryone
          on={everyone}
          count={plugins.length}
          risky={plugins.filter(runsByItself).length}
          words={words.everyone}
          onChange={(on) => forEveryone(undefined, on)}
        />
      )}
      {plugins !== null && plugins.length > SEARCH_FROM && (
        <label className="relative">
          <span className="sr-only">{words.search}</span>
          <Search
            aria-hidden
            size={15}
            className="pointer-events-none absolute top-1/2 left-3.5 -translate-y-1/2 text-muted"
          />
          <input
            type="search"
            value={query}
            placeholder={words.search}
            onChange={(event) => setQuery(event.target.value)}
            className="h-9 w-full rounded-full border border-transparent bg-sunken pr-4 pl-9 text-ink text-sm outline-none transition-colors placeholder:text-muted focus:border-line-strong focus:bg-panel"
          />
        </label>
      )}
      {plugins !== null && plugins.length > 0 && shown.length === 0 && (
        <p className="text-muted text-sm">{words.nothingFound(query.trim())}</p>
      )}
      {shown.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2">
          {shown.map((plugin) => (
            <li key={plugin.name}>
              <button
                type="button"
                onClick={() => setOpen(plugin.name)}
                className="flex h-full w-full flex-col gap-1.5 rounded-xl bg-bubble px-3 py-2.5 text-left hover:bg-sunken"
              >
                <span className="flex items-center gap-2">
                  <Puzzle aria-hidden size={14} className="shrink-0 text-muted" />
                  <span className="min-w-0 flex-1 truncate font-medium text-sm">{plugin.name}</span>
                  {warningsOf(plugin, words.warnings).length > 0 && (
                    <TriangleAlert
                      aria-label={words.warnings.title}
                      size={13}
                      className="shrink-0 text-warn"
                    />
                  )}
                </span>
                <span className="line-clamp-2 text-ink-soft text-xs">
                  {plugin.description || words.noDescription}
                </span>
                <span className="text-muted text-xs">
                  {words.source[plugin.source]} · {words.skills(plugin.skills.length)} ·{" "}
                  {everyone || plugin.everyone
                    ? words.everyone.one
                    : words.agents(plugin.botIds.length)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {opened && (
        <PluginDialog
          plugin={opened}
          allForEveryone={everyone}
          onShare={(botIds) => share(opened.name, botIds)}
          onEveryone={(on) => forEveryone(opened.name, on)}
          onClose={() => setOpen(null)}
        />
      )}
    </div>
  );
}
