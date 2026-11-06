// Settings > Skills (spec 38.4): the skills of the owner's own Claude Code,
// each a card with what it does, where it comes from and how many agents
// use it; the card opens who may use it, a switch per agent on Claude Code.

import { Sparkles, TriangleAlert } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Skill } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Dialog } from "../../ui/Dialog";
import { AgentSwitches } from "../connections/AgentSwitches";
import { SkillWarnings, warningsOf } from "./SkillWarnings";
import { useSkills } from "./useSkills";

function SkillDialog({
  skill,
  onShare,
  onClose,
}: {
  skill: Skill;
  onShare(botIds: string[]): unknown;
  onClose(): void;
}) {
  const words = useT().skills;
  const bots = useApp(
    useShallow((state) =>
      Object.values(state.bots).filter((bot) => bot.archivedAt === null && bot.agent === "claude"),
    ),
  );
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
          <AgentSwitches
            bots={bots}
            uses={(bot) => skill.botIds.includes(bot.id)}
            onChange={(bot, on) =>
              onShare(on ? [...skill.botIds, bot.id] : skill.botIds.filter((id) => id !== bot.id))
            }
            label={(bot) => words.canUse(bot.name)}
            empty={words.noAgents}
          />
        </section>
      </div>
    </Dialog>
  );
}

export function SkillsPage() {
  const t = useT();
  const words = t.skills;
  const { skills, share } = useSkills(words.shareFailed);
  const [open, setOpen] = useState<string | null>(null);
  const opened = skills?.find((skill) => skill.name === open);
  return (
    <div className="flex flex-col gap-4">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      {skills !== null && skills.length === 0 && (
        <p className="text-muted text-sm leading-relaxed">{words.none}</p>
      )}
      {skills !== null && skills.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2">
          {skills.map((skill) => (
            <li key={skill.name}>
              <button
                type="button"
                onClick={() => setOpen(skill.name)}
                className="flex h-full w-full flex-col gap-1.5 rounded-xl bg-bubble px-3 py-2.5 text-left hover:bg-sunken"
              >
                <span className="flex items-center gap-2">
                  <Sparkles aria-hidden size={14} className="shrink-0 text-muted" />
                  <span className="min-w-0 flex-1 truncate font-medium text-sm">{skill.name}</span>
                  {warningsOf(skill, words.warnings).length > 0 && (
                    <TriangleAlert
                      aria-label={words.warnings.title}
                      size={13}
                      className="shrink-0 text-warn"
                    />
                  )}
                </span>
                <span className="line-clamp-2 text-ink-soft text-xs">
                  {skill.description || words.noDescription}
                </span>
                <span className="text-muted text-xs">
                  {words.source[skill.source]} · {words.agents(skill.botIds.length)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {opened && (
        <SkillDialog
          skill={opened}
          onShare={(botIds) => share(opened.name, botIds)}
          onClose={() => setOpen(null)}
        />
      )}
    </div>
  );
}
