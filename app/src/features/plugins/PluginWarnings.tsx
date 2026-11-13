// What the owner should know before giving a plugin (spec 41.5): that it
// runs commands by itself, brings programs, leaves its connected tools out
// and asks for settings Botloft does not pass on.

import { TriangleAlert } from "lucide-react";
import { useT } from "../../i18n";
import type { Plugin } from "../../lib/protocol.gen";

type Words = ReturnType<typeof useT>["plugins"]["warnings"];

/** It runs commands by itself: hooks or language servers. */
export function runsByItself(plugin: Plugin): boolean {
  return plugin.hooks.length > 0 || plugin.languageServers.length > 0;
}

interface Warning {
  text: string;
  /** The technical names, in small print. */
  detail?: string;
}

export function warningsOf(plugin: Plugin, words: Words): Warning[] {
  const found: Warning[] = [];
  if (runsByItself(plugin)) {
    found.push({
      text: words.runs,
      detail: words.detail([...plugin.hooks, ...plugin.languageServers].join(", ")),
    });
  }
  if (plugin.programs) {
    found.push({ text: words.programs });
  }
  if (plugin.mcpServers.length > 0) {
    found.push({ text: words.tools(plugin.mcpServers.join(", ")) });
  }
  if (plugin.needsSettings) {
    found.push({ text: words.settings });
  }
  return found;
}

export function PluginWarnings({ plugin }: { plugin: Plugin }) {
  const words = useT().plugins.warnings;
  const found = warningsOf(plugin, words);
  if (found.length === 0) {
    return null;
  }
  return (
    <section className="flex flex-col gap-1.5 rounded-xl bg-bubble px-3 py-2">
      <h3 className="flex items-center gap-1.5 font-semibold text-xs">
        <TriangleAlert aria-hidden size={13} className="text-warn" />
        {words.title}
      </h3>
      <ul className="flex list-disc flex-col gap-1 pl-5 text-xs">
        {found.map((each) => (
          <li key={each.text} className="break-words" data-selectable>
            {each.text}
            {each.detail && <span className="block text-muted">{each.detail}</span>}
          </li>
        ))}
      </ul>
    </section>
  );
}
