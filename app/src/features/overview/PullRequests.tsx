// The agent's pull requests as GitHub has them now (spec 33.5): title,
// state, size and how the checks went, read through the owner's `gh`.

import { ExternalLink } from "lucide-react";
import { useT } from "../../i18n";
import type { GithubAccess, PullChecks, PullRequest, PullState } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { attempt } from "../../ui/toast";
import type { Tone } from "../../ui/tone";
import { EditCounts } from "../chat/EditDiff";

const STATE_TONES: Record<PullState, Tone> = {
  open: "work",
  draft: "quiet",
  merged: "ok",
  closed: "danger",
};

export function PullRequests({ prs, github }: { prs: PullRequest[]; github: GithubAccess | null }) {
  const w = useT().overview.changes;
  if (prs.length === 0 && (github === null || github === "ok")) {
    return null;
  }
  return (
    <div className="flex flex-col gap-1.5">
      <p className="font-medium text-sm">{w.prs}</p>
      {prs.length > 0 && (
        <ul aria-label={w.prs} className="flex flex-col gap-1.5">
          {prs.map((pr) => (
            <Pull key={pr.url} pr={pr} />
          ))}
        </ul>
      )}
      {github && github !== "ok" && <p className="text-muted text-xs">{w.github[github]}</p>}
    </div>
  );
}

function Pull({ pr }: { pr: PullRequest }) {
  const w = useT().overview.changes;
  const host = useHost();
  return (
    <li className="flex flex-col gap-1 rounded-xl bg-bubble p-2.5">
      <div className="flex items-start gap-2">
        <span className="min-w-0 flex-1 break-words font-medium text-sm leading-snug">
          {pr.title || w.pr(pr.number, null)}
        </span>
        <button
          type="button"
          title={pr.url}
          aria-label={w.openPr(pr.number)}
          onClick={() => attempt(w.openFailed, () => host.openUrl(pr.url))}
          className="grid size-6 shrink-0 place-items-center rounded-md text-muted hover:bg-sunken hover:text-ink"
        >
          <ExternalLink aria-hidden size={13} />
        </button>
      </div>
      <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs">
        <Badge tone={STATE_TONES[pr.state]}>{w.prState[pr.state]}</Badge>
        <span className="text-muted">#{pr.number}</span>
        <EditCounts lines={{ added: pr.additions, removed: pr.deletions }} />
        <Checks checks={pr.checks} />
      </div>
    </li>
  );
}

/** One line for the checks: the failures first, then what still runs. */
function Checks({ checks }: { checks: PullChecks }) {
  const w = useT().overview.changes.checks;
  const total = checks.passed + checks.failed + checks.pending;
  if (total === 0) {
    return <span className="text-muted">{w.none}</span>;
  }
  if (checks.failed > 0) {
    return <span className="text-danger">{w.failed(checks.failed, total)}</span>;
  }
  if (checks.pending > 0) {
    return <span className="text-work">{w.running(checks.pending, total)}</span>;
  }
  return <span className="text-ok">{w.passed(total)}</span>;
}
