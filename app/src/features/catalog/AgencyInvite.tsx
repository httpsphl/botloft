import { useState } from "react";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { AgencyGrid } from "./AgencyGrid";
import { BotAgencyDialog } from "./BotAgencyDialog";

/** The few roles the page shows first; the window has them all. */
export const FEATURED = [
  "developer",
  "designer",
  "writer",
  "researcher",
  "social-media",
  "customer-support",
  "personal-assistant",
  "project-manager",
] as const;

/**
 * For a crew with only its chief, or nobody: the owner often does not know
 * what to create, so a few bots of the Bot agency are on the page itself,
 * with a button for the rest (spec 26.5).
 */
export function AgencyInvite({ crew }: { crew: Crew }) {
  const t = useT();
  const [all, setAll] = useState(false);
  return (
    <section aria-label={t.catalog.title} className="mt-6 flex flex-col gap-3">
      <div>
        <h2 className="font-semibold text-base tracking-tight">{t.catalog.invite.title}</h2>
        <p className="mt-0.5 text-muted text-sm">{t.catalog.invite.body}</p>
      </div>
      <AgencyGrid crew={crew} featured={FEATURED} onSeeAll={() => setAll(true)} />
      {all && <BotAgencyDialog crew={crew} onClose={() => setAll(false)} />}
    </section>
  );
}
