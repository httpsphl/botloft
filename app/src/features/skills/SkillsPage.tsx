// Settings > Skills (spec 38.4): the skills of the owner's own Claude Code,
// each a card with what it does, where it comes from and how many agents
// use it; the card opens who may use it. Every skill can go to every agent
// at once, and a search finds one among many.

import { Search, Sparkles, TriangleAlert } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { plain } from "../account/settingsIndex";
import { AllForEveryone } from "./AllForEveryone";
import { SkillDialog } from "./SkillDialog";
import { warningsOf } from "./SkillWarnings";
import { useSkills } from "./useSkills";

/** With more skills than this, the page has a search. */
const SEARCH_FROM = 6;

export function SkillsPage() {
  const t = useT();
  const words = t.skills;
  const { skills, everyone, share, forEveryone } = useSkills(
    words.shareFailed,
    words.everyone.failed,
  );
  const [open, setOpen] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const opened = skills?.find((skill) => skill.name === open);
  const wanted = plain(query);
  const shown =
    skills?.filter(
      (skill) =>
        wanted === "" ||
        plain(skill.name).includes(wanted) ||
        plain(skill.description).includes(wanted),
    ) ?? [];
  return (
    <div className="flex flex-col gap-4">
      <p className="text-ink-soft text-sm leading-relaxed">{words.intro}</p>
      {skills !== null && skills.length === 0 && (
        <p className="text-muted text-sm leading-relaxed">{words.none}</p>
      )}
      {skills !== null && skills.length > 0 && (
        <AllForEveryone
          on={everyone}
          count={skills.length}
          risky={skills.filter((skill) => warningsOf(skill, words.warnings).length > 0).length}
          words={words.everyone}
          onChange={(on) => forEveryone(undefined, on)}
        />
      )}
      {skills !== null && skills.length > SEARCH_FROM && (
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
      {skills !== null && skills.length > 0 && shown.length === 0 && (
        <p className="text-muted text-sm">{words.nothingFound(query.trim())}</p>
      )}
      {shown.length > 0 && (
        <ul className="grid gap-2 sm:grid-cols-2">
          {shown.map((skill) => (
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
                  {words.source[skill.source]} ·{" "}
                  {everyone || skill.everyone
                    ? words.everyone.one
                    : words.agents(skill.botIds.length)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {opened && (
        <SkillDialog
          skill={opened}
          allForEveryone={everyone}
          onShare={(botIds) => share(opened.name, botIds)}
          onEveryone={(on) => forEveryone(opened.name, on)}
          onClose={() => setOpen(null)}
        />
      )}
    </div>
  );
}
