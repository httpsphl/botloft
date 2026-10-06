import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { AgencyGrid } from "./AgencyGrid";

/**
 * For a crew with only its chief, or nobody: the owner often does not know
 * what to create, so the Bot agency is on the page itself (spec 26.5).
 */
export function AgencyInvite({ crew }: { crew: Crew }) {
  const t = useT();
  return (
    <section aria-label={t.catalog.title} className="mt-6 flex flex-col gap-3">
      <div>
        <h2 className="font-semibold text-base tracking-tight">{t.catalog.invite.title}</h2>
        <p className="mt-0.5 text-muted text-sm">{t.catalog.invite.body}</p>
      </div>
      <AgencyGrid crew={crew} />
    </section>
  );
}
