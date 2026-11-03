// Bringing the owner's claude.ai export to the Maker (spec 36.6, 36.7):
// three steps to get the ZIP from claude.ai (Botloft never signs in there
// nor reads the e-mail), choosing the file, which the daemon reads where
// it is, a period, then the same summary to read before it goes.

import { ExternalLink, FileArchive } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ImportConversations } from "../../lib/protocol.gen";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Choices } from "../../ui/Choices";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { DraftReview, period, useImportDraft } from "./importParts";

/** Where claude.ai's settings are; "Privacy" holds "Export data" (spec 36.6). */
const CLAUDE_SETTINGS = "https://claude.ai/settings";
const DAY_MS = 24 * 60 * 60 * 1000;

type Period = "all" | "year" | "half" | "quarter";
const DAYS: Record<Period, number | null> = { all: null, year: 365, half: 182, quarter: 91 };

/** The dialog that brings the owner's claude.ai export to the Maker. */
export function ClaudeAiImport({ onClose }: { onClose(): void }) {
  const t = useT().maker;
  const a = t.claudeAi;
  const api = useApi();
  const host = useHost();
  const [path, setPath] = useState<string | null>(null);
  const [found, setFound] = useState<ImportConversations | null>(null);
  const [span, setSpan] = useState<Period>("all");
  const { draft, busy, setBusy, make, forget, footer } = useImportDraft(onClose);

  const pick = async () => {
    const picked = await host.pickFile(a.pickTitle, a.kind, ["zip"]);
    if (!picked) return;
    setBusy(true);
    setFound(null);
    await attempt(a.scanFailed, async () => {
      const scan = await api.call("imports.scan", { source: "claude_ai", path: picked });
      setPath(picked);
      setFound(scan.conversations);
    });
    setBusy(false);
  };

  const build = () => {
    if (!path) return;
    const days = DAYS[span];
    void make({
      source: "claude_ai",
      keys: [],
      path,
      ...(days !== null && { from: Date.now() - days * DAY_MS }),
    });
  };

  return (
    <Dialog
      title={a.title}
      onClose={draft ? forget : onClose}
      width="lg"
      footer={
        footer || (
          <Button variant="primary" onClick={build} disabled={busy || !found || found.count === 0}>
            {t.history.make}
          </Button>
        )
      }
    >
      <div className="flex flex-col gap-4 text-sm">
        {!draft && (
          <>
            <p className="text-ink-soft leading-relaxed">{a.intro}</p>
            <ol className="flex list-decimal flex-col gap-1.5 pl-5 text-ink-soft">
              {a.steps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
            <div className="flex flex-wrap gap-2">
              <Button icon={ExternalLink} onClick={() => void host.openUrl(CLAUDE_SETTINGS)}>
                {a.openSite}
              </Button>
              <Button icon={FileArchive} onClick={pick} disabled={busy}>
                {found ? a.other : a.pick}
              </Button>
            </div>
            <p className="text-muted text-xs leading-relaxed">{a.takes}</p>
          </>
        )}
        {busy && !draft && <p className="text-muted">{t.history.making}</p>}
        {!draft && found && (
          <div className="flex flex-col gap-3 rounded-xl border border-line px-4 py-3">
            <p className="font-medium">
              {a.found(found.count, period(found.firstAt, found.lastAt))}
            </p>
            {found.projects.length > 0 && (
              <p className="text-ink-soft text-xs">{a.projects(found.projects.join(", "))}</p>
            )}
            <Choices<Period>
              label={a.periodLabel}
              value={span}
              options={(["all", "year", "half", "quarter"] as const).map((value) => ({
                value,
                label: a.periods[value],
              }))}
              onChange={setSpan}
            />
          </div>
        )}
        {draft && (
          <DraftReview draft={draft} cut={draft.cut.length > 0 ? a.cut(draft.cut.length) : null} />
        )}
      </div>
    </Dialog>
  );
}
