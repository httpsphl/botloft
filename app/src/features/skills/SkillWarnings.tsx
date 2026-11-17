// What the owner should know before giving a skill (spec 38.1): the tools
// it runs without asking, the commands it runs by itself, its other files.

import { TriangleAlert } from "lucide-react";
import { useT } from "../../i18n";
import type { Skill } from "../../lib/protocol.gen";

export function warningsOf(skill: Skill, words: ReturnType<typeof useT>["skills"]["warnings"]) {
  const found: string[] = [];
  if (skill.allowedTools.length > 0) {
    found.push(words.tools(skill.allowedTools.join(", ")));
  }
  if (skill.runsCommands) {
    found.push(words.commands);
  }
  if (skill.hasFiles) {
    found.push(words.files);
  }
  return found;
}

export function SkillWarnings({ skill }: { skill: Skill }) {
  const words = useT().skills.warnings;
  const found = warningsOf(skill, words);
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
          <li key={each} className="break-words" data-selectable>
            {each}
          </li>
        ))}
      </ul>
    </section>
  );
}
