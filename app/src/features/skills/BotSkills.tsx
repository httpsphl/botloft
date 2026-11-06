// The owner's skills in one agent's details (spec 38.4): the ones it has,
// and the way to Settings > Skills. Only agents on Claude Code.

import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { useSkills } from "./useSkills";

export function BotSkills({ bot }: { bot: Bot }) {
  const words = useT().skills;
  const { skills } = useSkills(words.shareFailed);
  const openSettingsPart = useApp((state) => state.openSettingsPart);
  if (bot.agent !== "claude" || skills === null || skills.length === 0) {
    return null;
  }
  const mine = skills.filter((skill) => skill.botIds.includes(bot.id));
  return (
    <div>
      <dt className="text-muted text-xs">{words.details.label}</dt>
      <dd className="mt-1 flex flex-col gap-1.5">
        {mine.length === 0 ? (
          <p className="text-ink-soft text-xs leading-relaxed">{words.details.none(bot.name)}</p>
        ) : (
          <p className="text-xs" data-selectable>
            {mine.map((skill) => skill.name).join(", ")}
          </p>
        )}
        <Button size="sm" className="self-start" onClick={() => openSettingsPart("skills")}>
          {words.details.choose}
        </Button>
      </dd>
    </div>
  );
}
