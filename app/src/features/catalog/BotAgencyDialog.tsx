import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { Dialog } from "../../ui/Dialog";
import { AgencyGrid } from "./AgencyGrid";

/** The Bot agency, opened from a crew's page. */
export function BotAgencyDialog({ crew, onClose }: { crew: Crew; onClose(): void }) {
  const t = useT();
  return (
    <Dialog title={t.catalog.title} width="lg" onClose={onClose}>
      <p className="mb-3 text-muted text-sm">{t.catalog.intro}</p>
      <AgencyGrid crew={crew} onLeave={onClose} />
    </Dialog>
  );
}
