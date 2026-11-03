// What changed in the agent's git folders and what it did in git (spec
// 33.5). Read when the panel opens, after each reply and every minute
// while it is open.

import { ExternalLink, GitBranch } from "lucide-react";
import { useCallback, useEffect, useId, useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Bot, GitEvent, GitFile, GitRepo, OverviewChanges } from "../../lib/protocol.gen";
import { useApi, useHost } from "../../store/context";
import { attempt } from "../../ui/toast";
import { EditCounts } from "../chat/EditDiff";
import { PullRequests } from "./PullRequests";

const EVERY_MS = 60_000;
/** Files shown before "Show all". */
const FIRST = 8;

export function ChangesSection({ bot }: { bot: Bot }) {
  const api = useApi();
  const w = useT().overview.changes;
  const id = useId();
  const [changes, setChanges] = useState<OverviewChanges | null>(null);
  const load = useCallback(
    () => api.call("overview.changes", { botId: bot.id }).then(setChanges, () => setChanges(null)),
    [api, bot.id],
  );
  // biome-ignore lint/correctness/useExhaustiveDependencies: each reply may have changed files
  useEffect(() => {
    void load();
    const timer = setInterval(() => void load(), EVERY_MS);
    return () => clearInterval(timer);
  }, [load, bot.lastReplyAt]);

  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h3 id={id} className="font-semibold text-sm">
        {w.title}
      </h3>
      {changes === null ? (
        <p className="text-muted text-sm">{w.loading}</p>
      ) : (
        <>
          {changes.repos.length === 0 ? (
            <p className="text-muted text-sm">{w.noRepo(bot.name)}</p>
          ) : (
            changes.repos.map((repo) => <Repo key={repo.path} repo={repo} />)
          )}
          <PullRequests prs={changes.prs} github={changes.github} />
          {changes.events.length > 0 && <Events bot={bot} events={changes.events} />}
        </>
      )}
    </section>
  );
}

function Repo({ repo }: { repo: GitRepo }) {
  const w = useT().overview.changes;
  const [all, setAll] = useState(false);
  const shown = all ? repo.files : repo.files.slice(0, FIRST);
  const hidden = repo.files.length - shown.length + repo.more;
  return (
    <div className="flex flex-col gap-1 rounded-xl bg-bubble p-2.5" title={repo.path}>
      <p className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-sm">
        <span className="font-semibold">{repo.name}</span>
        {repo.branch && (
          <span className="inline-flex items-center gap-1 text-muted text-xs">
            <GitBranch aria-hidden size={12} />
            {repo.branch}
          </span>
        )}
        {(repo.ahead > 0 || repo.behind > 0) && (
          <span className="text-muted text-xs tabular-nums">
            {w.aheadBehind(repo.ahead, repo.behind)}
          </span>
        )}
      </p>
      {repo.files.length === 0 && repo.more === 0 ? (
        <p className="text-muted text-xs">{w.clean}</p>
      ) : (
        <ul aria-label={w.files(repo.name)} className="flex flex-col">
          {shown.map((file) => (
            <File key={file.path} file={file} />
          ))}
        </ul>
      )}
      {hidden > 0 &&
        (repo.more > 0 && all ? (
          <p className="px-1 text-muted text-xs">{w.more(repo.more)}</p>
        ) : (
          <button
            type="button"
            onClick={() => setAll(true)}
            className="self-start rounded-md px-1 text-muted text-xs hover:text-ink"
          >
            {w.showAll(repo.files.length + repo.more)}
          </button>
        ))}
    </div>
  );
}

const MARK: Record<GitFile["status"], { mark: string; tone: string }> = {
  modified: { mark: "M", tone: "text-work" },
  added: { mark: "A", tone: "text-ok" },
  deleted: { mark: "D", tone: "text-danger" },
  renamed: { mark: "R", tone: "text-muted" },
  untracked: { mark: "+", tone: "text-ok" },
  conflicted: { mark: "!", tone: "text-warn" },
};

function File({ file }: { file: GitFile }) {
  const w = useT().overview.changes;
  const { mark, tone } = MARK[file.status];
  return (
    <li className="flex items-baseline gap-2 px-1 py-0.5 text-xs" title={w.status[file.status]}>
      <span aria-hidden className={`w-3 shrink-0 font-mono font-semibold ${tone}`}>
        {mark}
      </span>
      <span className="sr-only">{w.status[file.status]}:</span>
      <span className="min-w-0 flex-1 break-all font-mono" data-selectable>
        {file.path}
      </span>
      {file.added !== null && file.removed !== null ? (
        <EditCounts lines={{ added: file.added, removed: file.removed }} />
      ) : file.status === "untracked" ? (
        <span className="shrink-0 text-ok">{w.newFile}</span>
      ) : null}
    </li>
  );
}

function Events({ bot, events }: { bot: Bot; events: GitEvent[] }) {
  const w = useT().overview.changes;
  const host = useHost();
  return (
    <div className="flex flex-col gap-1.5">
      <p className="font-medium text-sm">{w.didTitle(bot.name)}</p>
      <ul aria-label={w.didTitle(bot.name)} className="flex flex-col gap-1">
        {events.map((event, index) => (
          // Events never move once listed: their place is their key.
          // biome-ignore lint/suspicious/noArrayIndexKey: a list read whole each time
          <li key={index} className="flex items-baseline gap-2 text-sm">
            <span className="min-w-0 flex-1 break-words">{describe(event, w)}</span>
            {event.url && (
              <button
                type="button"
                title={event.url}
                aria-label={w.openPr(event.number ?? 0)}
                onClick={() => attempt(w.openFailed, () => host.openUrl(event.url as string))}
                className="grid size-6 shrink-0 place-items-center self-center rounded-md text-muted hover:bg-sunken hover:text-ink"
              >
                <ExternalLink aria-hidden size={13} />
              </button>
            )}
            <span className="shrink-0 text-muted text-xs">{when(event.at)}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

type Words = ReturnType<typeof useT>["overview"]["changes"];

/** "Commit a1b2c3d on main", "PR #12 opened", in the owner's language. */
function describe(event: GitEvent, w: Words): string {
  switch (event.kind) {
    case "commit":
      return w.commit(event.sha ?? "", event.branch);
    case "push":
      return w.push(event.branch);
    case "branch":
      return event.action === "rebased" ? w.rebased(event.branch) : w.merged(event.branch);
    case "pr": {
      const action = event.action && event.action in w.prAction ? event.action : null;
      return w.pr(event.number ?? 0, action ? w.prAction[action as keyof Words["prAction"]] : null);
    }
  }
}
