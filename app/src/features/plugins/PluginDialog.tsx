// A plugin's dialog (spec 41.5): what it does and brings, what to know
// before giving it, and who may use it: every agent, or a switch per agent
// on Claude Code.

import { useId } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Plugin } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { AgentSwitches } from "../connections/AgentSwitches";
import { PluginWarnings } from "./PluginWarnings";

export function PluginDialog({
  plugin,
  allForEveryone,
  onShare,
  onEveryone,
  onClose,
}: {
  plugin: Plugin;
  /** Every plugin is on for every agent: this one cannot be changed here. */
  allForEveryone: boolean;
  onShare(botIds: string[]): unknown;
  onEveryone(on: boolean): unknown;
  onClose(): void;
}) {
  const words = useT().plugins;
  const id = useId();
  const bots = useApp(
    useShallow((state) =>
      Object.values(state.bots).filter((bot) => bot.archivedAt === null && bot.agent === "claude"),
    ),
  );
  const everyone = allForEveryone || plugin.everyone;
  const origin = [words.source[plugin.source], plugin.version && words.version(plugin.version)];
  return (
    <Dialog title={plugin.name} onClose={onClose}>
      <div className="flex flex-col gap-4">
        <p className="text-muted text-xs">{origin.filter(Boolean).join(" · ")}</p>
        <p className="text-ink-soft text-sm leading-relaxed" data-selectable>
          {plugin.description || words.noDescription}
        </p>
        {plugin.source === "kept" && <p className="text-muted text-xs">{words.kept}</p>}
        <section className="flex flex-col gap-1.5">
          <h3 className="font-semibold text-sm">{words.brings.title}</h3>
          {plugin.skills.length === 0 && plugin.agents.length === 0 && (
            <p className="text-muted text-xs">{words.brings.nothing}</p>
          )}
          <Names title={words.brings.skills} names={plugin.skills.map((name) => `/${name}`)} />
          <Names title={words.brings.agents} names={plugin.agents} />
        </section>
        <PluginWarnings plugin={plugin} />
        <section className="flex flex-col gap-2">
          <h3 className="font-semibold text-sm">{words.who}</h3>
          <div className="flex items-center gap-3 rounded-xl bg-bubble px-3.5 py-2.5">
            <div className="min-w-0 flex-1">
              <p id={id} className="text-sm">
                {words.everyone.one}
              </p>
              <p className="text-muted text-xs">
                {allForEveryone ? words.everyone.byAll : words.everyone.oneHint}
              </p>
            </div>
            <Switch
              checked={everyone}
              disabled={allForEveryone}
              labelledBy={id}
              onChange={(on) => onEveryone(on)}
            />
          </div>
          <AgentSwitches
            bots={bots}
            uses={(bot) => plugin.botIds.includes(bot.id)}
            onChange={(bot, on) =>
              onShare(
                on ? [...plugin.botIds, bot.id] : plugin.botIds.filter((each) => each !== bot.id),
              )
            }
            label={(bot) => words.canUse(bot.name)}
            empty={words.noAgents}
            locked={everyone}
          />
        </section>
      </div>
    </Dialog>
  );
}

/** A list of names as the agent calls them, in small code type. */
function Names({ title, names }: { title: string; names: string[] }) {
  if (names.length === 0) {
    return null;
  }
  return (
    <div className="flex flex-col gap-1 rounded-xl bg-bubble px-3 py-2">
      <p className="text-muted text-xs">{title}</p>
      <ul className="flex flex-wrap gap-1.5" aria-label={title}>
        {names.map((name) => (
          <li
            key={name}
            className="rounded-md bg-sunken px-1.5 py-0.5 font-mono text-xs"
            data-selectable
          >
            {name}
          </li>
        ))}
      </ul>
    </div>
  );
}
