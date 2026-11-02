// The Bot agency's roles (spec 26.5): filters, search and the roles in a
// section per kind, or one role in full after "Learn more". Used in the window the crew's button
// opens and, for a crew with only its chief, in the page itself, where
// `featured` shows just a few and a button to see them all.

import { Search } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { AVATAR_PALETTE, type BotTemplate, type Crew } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { AgencyCard } from "./AgencyCard";
import { RoleDetail } from "./RoleDetail";
import { roleText } from "./roleText";
import { useAddRole } from "./useAddRole";
import { useCatalog } from "./useCatalog";

type Category = BotTemplate["category"] | "all";

const CATEGORIES: Category[] = [
  "all",
  "product",
  "marketing",
  "code",
  "design",
  "content",
  "research",
  "business",
  "learning",
  "finance",
  "people",
  "security",
  "health",
  "games",
  "legal",
];

/** A fixed color per role, so the mascots tell the cards apart. */
const colorOf = (index: number) => AVATAR_PALETTE[index % AVATAR_PALETTE.length] ?? "#FF7A59";

export function AgencyGrid({
  crew,
  onLeave,
  featured,
  onSeeAll,
}: {
  crew: Crew;
  onLeave?: () => void;
  /** Role ids to show alone, in this order, without filters or search. */
  featured?: readonly string[];
  onSeeAll?: () => void;
}) {
  const t = useT();
  const { roles, failed } = useCatalog();
  const { add, added, busy } = useAddRole(crew);
  const selectBot = useApp((state) => state.selectBot);
  const setPanel = useApp((state) => state.setPanel);
  const [category, setCategory] = useState<Category>("all");
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState<string | null>(null);

  if (failed) {
    return null;
  }
  if (roles === null) {
    return <p className="text-muted text-sm">{t.catalog.loading}</p>;
  }

  const customize = (botId: string) => {
    selectBot(botId);
    setPanel(botId, "details");
    onLeave?.();
  };

  const opened = roles.find((role) => role.id === open);
  if (opened) {
    const bot = added[opened.id];
    return (
      <RoleDetail
        template={opened}
        text={roleText(t, opened)}
        color={colorOf(roles.indexOf(opened))}
        added={bot}
        busy={busy === opened.id}
        onAdd={() => add(opened)}
        onBack={() => setOpen(null)}
        onCustomize={() => bot && customize(bot.id)}
      />
    );
  }

  const needle = query.trim().toLowerCase();
  const shown = featured
    ? featured.flatMap((id) => roles.filter((role) => role.id === id))
    : roles.filter((role) => {
        if (category !== "all" && role.category !== category) {
          return false;
        }
        const text = roleText(t, role);
        return (
          !needle || `${text.name} ${text.summary} ${text.role}`.toLowerCase().includes(needle)
        );
      });
  // Only the kinds that have a role to show.
  const kinds = CATEGORIES.filter(
    (id) => id === "all" || roles.some((role) => role.category === id),
  );

  const card = (role: BotTemplate, withKind: boolean) => {
    const bot = added[role.id];
    return (
      <AgencyCard
        key={role.id}
        template={role}
        text={roleText(t, role)}
        color={colorOf(roles.indexOf(role))}
        withKind={withKind}
        added={bot}
        busy={busy === role.id}
        onAdd={() => add(role)}
        onLearnMore={() => setOpen(role.id)}
        onCustomize={() => bot && customize(bot.id)}
      />
    );
  };
  const list = "grid grid-cols-[repeat(auto-fill,minmax(19rem,1fr))] gap-x-4 gap-y-1";

  if (featured) {
    return (
      <div className="flex flex-col gap-3">
        <ul className={`-mx-2 ${list}`}>{shown.map((role) => card(role, true))}</ul>
        {onSeeAll && (
          <div>
            <Button onClick={onSeeAll}>{t.catalog.seeAll}</Button>
          </div>
        )}
      </div>
    );
  }

  // One section per kind, in the filters' order: its name is said once, on
  // top, and the lines have the room for theirs.
  const sections = kinds
    .filter((id): id is BotTemplate["category"] => id !== "all")
    .map((id) => ({ id, roles: shown.filter((role) => role.category === id) }))
    .filter((section) => section.roles.length > 0);

  return (
    <div className="flex flex-col gap-5">
      {/* Stays on top while the list scrolls under it. */}
      <div className="sticky top-0 z-10 -mx-6 -mt-1 flex flex-col gap-2.5 bg-panel px-6 pt-1 pb-3">
        <label className="relative block">
          <Search
            aria-hidden
            size={15}
            className="pointer-events-none absolute top-1/2 left-3.5 -translate-y-1/2 text-muted"
          />
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            aria-label={t.catalog.search}
            placeholder={t.catalog.search}
            className="h-10 w-full rounded-full border border-transparent bg-sunken pr-4 pl-10 text-ink text-sm outline-none transition-colors placeholder:text-muted focus:border-line-strong focus:bg-panel"
          />
        </label>
        <fieldset
          aria-label={t.catalog.filter}
          className="m-0 flex min-w-0 flex-wrap gap-1.5 border-0 p-0"
        >
          {kinds.map((id) => (
            <button
              key={id}
              type="button"
              aria-pressed={category === id}
              onClick={() => setCategory(id)}
              className={`h-7 rounded-full border px-2.5 font-medium text-xs transition-colors ${
                category === id
                  ? "border-ink bg-ink text-canvas"
                  : "border-line-strong text-ink-soft hover:bg-sunken hover:text-ink"
              }`}
            >
              {t.catalog.categories[id]}
            </button>
          ))}
        </fieldset>
      </div>
      {sections.length === 0 ? (
        <p className="py-6 text-center text-muted text-sm">{t.catalog.none}</p>
      ) : (
        sections.map((section) => {
          const headingId = `agency-${section.id}`;
          return (
            <section key={section.id} aria-labelledby={headingId}>
              <h3 id={headingId} className="mb-1.5 flex items-baseline gap-2 font-semibold text-sm">
                {t.catalog.categories[section.id]}
                <span className="font-normal text-muted text-xs tabular-nums">
                  {section.roles.length}
                </span>
              </h3>
              <ul className={`-mx-2 ${list}`}>{section.roles.map((role) => card(role, false))}</ul>
            </section>
          );
        })
      )}
    </div>
  );
}
