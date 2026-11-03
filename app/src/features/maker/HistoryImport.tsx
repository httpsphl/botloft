// Bringing the owner's Claude Code history to the Maker (spec 36.5, 36.7):
// choose the projects, read the whole summary the daemon made (what was
// swapped out, how much of the plan it takes), then send it to the Maker
// or forget it. Nothing leaves before the owner clicks Send.

import { useCallback, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { ImportProject } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { ImportFailed, ImportWorking } from "./ImportWorking";
import { Check, DraftReview, period, useImportDraft } from "./importParts";

/** The dialog that brings the owner's Claude Code history to the Maker. */
export function HistoryImport({ onClose }: { onClose(): void }) {
  const h = useT().maker.history;
  const api = useApi();
  const [projects, setProjects] = useState<ImportProject[] | null>(null);
  const [failed, setFailed] = useState(false);
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  /** The projects the summary was made from, to take some out again. */
  const [made, setMade] = useState<Set<string>>(new Set());
  const { draft, busy, make, forget, footer } = useImportDraft(onClose);
  const making = busy && !draft;

  /** Lists the history again: the first reading with a cold disk is slow. */
  const scan = useCallback(
    async (live: () => boolean) => {
      setFailed(false);
      setProjects(null);
      const ok = await attempt(h.scanFailed, async () => {
        const found = await api.call("imports.scan", { source: "claude_code" });
        if (!live()) return;
        setProjects(found.projects);
        setChosen(new Set(found.projects.map((project) => project.key)));
      });
      if (!ok && live()) setFailed(true);
    },
    [api, h.scanFailed],
  );

  useEffect(() => {
    let live = true;
    void scan(() => live);
    return () => {
      live = false;
    };
  }, [scan]);

  const toggle = (key: string, on: boolean) =>
    setChosen((now) => {
      const next = new Set(now);
      if (on) next.add(key);
      else next.delete(key);
      return next;
    });

  const build = async () => {
    const keys = [...chosen];
    if (await make({ source: "claude_code", keys })) setMade(new Set(keys));
  };

  return (
    <Dialog
      title={h.title}
      onClose={draft ? forget : onClose}
      width="lg"
      footer={
        footer || (
          <Button
            variant="primary"
            onClick={build}
            disabled={busy || chosen.size === 0 || !projects?.length}
          >
            {h.make}
          </Button>
        )
      }
    >
      <div className="flex flex-col gap-4 text-sm">
        {!draft && (
          <>
            <p className="text-ink-soft leading-relaxed">{h.intro}</p>
            <p className="text-muted text-xs leading-relaxed">{h.takes}</p>
          </>
        )}
        {failed && <ImportFailed text={h.scanError} onRetry={() => void scan(() => true)} />}
        {projects === null && !failed && <ImportWorking title={h.loading} hint={h.loadingHint} />}
        {projects?.length === 0 && <p className="text-muted">{h.none}</p>}
        {making && <ImportWorking title={h.making} hint={h.makingHint} />}
        {!draft && !making && projects && projects.length > 0 && (
          <div className="flex flex-col">
            <Check
              label={h.all}
              checked={chosen.size === projects.length}
              onChange={(on) =>
                setChosen(on ? new Set(projects.map((project) => project.key)) : new Set())
              }
            />
            <div className="ml-6 flex flex-col">
              {projects.map((project) => (
                <Check
                  key={project.key}
                  label={project.path}
                  hint={h.project(
                    project.sessions,
                    project.prompts,
                    period(project.firstAt, project.lastAt),
                  )}
                  checked={chosen.has(project.key)}
                  onChange={(on) => toggle(project.key, on)}
                />
              ))}
            </div>
          </div>
        )}
        {draft && (
          <DraftReview
            draft={draft}
            cut={draft.cut.length > 0 ? `${h.cut}: ${draft.cut.join(", ")}` : null}
          >
            <details className="rounded-xl border border-line px-3 py-2">
              <summary className="cursor-pointer font-medium">{h.projects}</summary>
              <div className="mt-2 flex flex-col">
                {(projects ?? [])
                  .filter((project) => made.has(project.key))
                  .map((project) => (
                    <Check
                      key={project.key}
                      label={project.path}
                      checked={chosen.has(project.key)}
                      onChange={(on) => toggle(project.key, on)}
                    />
                  ))}
                <div className="mt-2">
                  <Button size="sm" onClick={build} disabled={busy || chosen.size === 0}>
                    {h.remake}
                  </Button>
                </div>
              </div>
            </details>
          </DraftReview>
        )}
      </div>
    </Dialog>
  );
}
