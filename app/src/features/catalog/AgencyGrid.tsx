// The Bot agency's roles (spec 26.5): filters, search and the cards, or one
// role in full after "Learn more". Used in the window the crew's button
// opens and, for a crew with only its chief, in the page itself, where
// `featured` shows just a few and a button to see them all.

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

  return (
    <div className="flex flex-col gap-3">
      {!featured && (
        <div className="flex flex-wrap items-center gap-2">
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            aria-label={t.catalog.search}
            placeholder={t.catalog.search}
            className="h-8 w-48 rounded-lg border border-line-strong bg-canvas px-2.5 text-ink text-sm outline-none placeholder:text-muted focus:border-accent"
          />
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
      )}
      {shown.length === 0 ? (
        <p className="py-6 text-center text-muted text-sm">{t.catalog.none}</p>
      ) : (
        <ul className="grid grid-cols-[repeat(auto-fill,minmax(230px,1fr))] gap-3">
          {shown.map((role) => {
            const bot = added[role.id];
            return (
              <AgencyCard
                key={role.id}
                template={role}
                text={roleText(t, role)}
                color={colorOf(roles.indexOf(role))}
                added={bot}
                busy={busy === role.id}
                onAdd={() => add(role)}
                onLearnMore={() => setOpen(role.id)}
                onCustomize={() => bot && customize(bot.id)}
              />
            );
          })}
        </ul>
      )}
      {featured && onSeeAll && (
        <div>
          <Button onClick={onSeeAll}>{t.catalog.seeAll}</Button>
        </div>
      )}
    </div>
  );
}
