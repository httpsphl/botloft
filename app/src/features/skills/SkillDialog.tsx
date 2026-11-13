// A skill's dialog (spec 38.4): what it does, what to know before giving
// it, and who may use it: every agent, or a switch per agent on Claude Code.

import { useId } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Skill } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Dialog } from "../../ui/Dialog";
import { Switch } from "../../ui/Switch";
import { AgentSwitches } from "../connections/AgentSwitches";
import { SkillWarnings } from "./SkillWarnings";

export function SkillDialog({
  skill,
  allForEveryone,
  onShare,
  onEveryone,
  onClose,
}: {
  skill: Skill;
  /** Every skill is on for every agent: this one cannot be changed here. */
  allForEveryone: boolean;
  onShare(botIds: string[]): unknown;
  onEveryone(on: boolean): unknown;
  onClose(): void;
}) {
  const words = useT().skills;
  const id = useId();
  const bots = useApp(
    useShallow((state) =>
      Object.values(state.bots).filter((bot) => bot.archivedAt === null && bot.agent === "claude"),
    ),
  );
  const everyone = allForEveryone || skill.everyone;
  return (
    <Dialog title={skill.name} onClose={onClose}>
      <div className="flex flex-col gap-4">
        <p className="text-muted text-xs">{words.source[skill.source]}</p>
        <p className="text-ink-soft text-sm leading-relaxed" data-selectable>
          {skill.description || words.noDescription}
        </p>
        {skill.source === "kept" && <p className="text-muted text-xs">{words.kept}</p>}
        <SkillWarnings skill={skill} />
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
            uses={(bot) => skill.botIds.includes(bot.id)}
            onChange={(bot, on) =>
              onShare(
                on ? [...skill.botIds, bot.id] : skill.botIds.filter((each) => each !== bot.id),
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
