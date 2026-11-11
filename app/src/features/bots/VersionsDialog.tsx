// The history of an agent's memory or instructions (spec 8.10): every
// version, newest first, with who made it and when. Each opens to what it
// changed, and an older one can come back, itself as a new version.

import { ChevronRight, RotateCcw } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Bot, TextVersion, VersionKind } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { EditCounts, EditDiff } from "../chat/EditDiff";
import { diffLines } from "../chat/lineDiff";

type Words = Messages["versions"];

/** The bot's versions of `kind`, kept up to date as new ones come. */
function useVersions(botId: string, kind: VersionKind, words: Words): TextVersion[] | null {
  const api = useApi();
  const [versions, setVersions] = useState<TextVersion[] | null>(null);
  useEffect(() => {
    let alive = true;
    attempt(words.failed.load, async () => {
      const list = await api.call("versions.list", { botId, kind });
      if (alive) {
        setVersions(list);
      }
    });
    const off = api.subscribe((event) => {
      if (
        event.name === "version.created" &&
        event.params.botId === botId &&
        event.params.kind === kind
      ) {
        const fresh = event.params;
        setVersions((list) =>
          list && !list.some((have) => have.id === fresh.id) ? [fresh, ...list] : list,
        );
      }
    });
    return () => {
      alive = false;
      off();
    };
  }, [api, botId, kind, words]);
  return versions;
}

function authorOf(version: TextVersion, all: TextVersion[], bot: Bot, words: Words): string {
  switch (version.author) {
    case "start":
      return words.author.start;
    case "owner": {
      const back = all.find((have) => have.id === version.restoredFrom);
      return back ? words.author.restored(when(back.createdAt)) : words.author.owner;
    }
    case "agent":
      return words.author.agent(bot.name);
    case "maker":
      return words.author.maker;
    case "outside":
      return words.author.outside;
  }
}

function VersionRow({
  version,
  older,
  newest,
  who,
  onRestore,
}: {
  version: TextVersion;
  /** The version before it; none for the oldest. */
  older: TextVersion | undefined;
  newest: boolean;
  who: string;
  onRestore(): void;
}) {
  const words = useT().versions;
  const [open, setOpen] = useState(false);
  const diff = useMemo(() => diffLines(older?.text ?? "", version.text), [older, version.text]);
  const lines = {
    added: diff.filter((line) => line.kind === "added").length,
    removed: diff.filter((line) => line.kind === "removed").length,
  };
  return (
    <li className="flex flex-col gap-2 rounded-xl bg-bubble px-3 py-2">
      <div className="flex min-w-0 items-center gap-2">
        <button
          type="button"
          aria-expanded={open}
          aria-label={`${who}, ${when(version.createdAt)}: ${words.showChanges}`}
          onClick={() => setOpen(!open)}
          className="flex min-w-0 flex-1 items-center gap-2 text-left text-sm"
        >
          <ChevronRight
            aria-hidden
            size={13}
            className={`shrink-0 text-muted transition-transform ${open ? "rotate-90" : ""}`}
          />
          <span className="min-w-0 flex-1 truncate font-medium">{who}</span>
          <span className="shrink-0 text-muted text-xs">{when(version.createdAt)}</span>
          <EditCounts lines={lines} />
        </button>
        {newest && <Badge tone="quiet">{words.now}</Badge>}
      </div>
      {open && (
        <div className="flex animate-rise flex-col gap-2" data-selectable>
          {version.text === "" ? (
            <p className="text-muted text-sm">{words.emptyText}</p>
          ) : (
            <EditDiff diffs={[diff]} />
          )}
          {!newest && (
            <Button size="sm" icon={RotateCcw} onClick={onRestore} className="self-start">
              {words.restore}
            </Button>
          )}
        </div>
      )}
    </li>
  );
}

export function VersionsDialog({
  bot,
  kind,
  onClose,
}: {
  bot: Bot;
  kind: VersionKind;
  onClose(): void;
}) {
  const words = useT().versions;
  const api = useApi();
  const versions = useVersions(bot.id, kind, words);
  const [restoring, setRestoring] = useState<TextVersion | null>(null);
  return (
    <>
      <Dialog title={words.title[kind](bot.name)} onClose={onClose} width="lg">
        <div className="flex flex-col gap-3">
          <p className="text-muted text-sm">{words.intro[kind](bot.name)}</p>
          {versions === null ? null : versions.length === 0 ? (
            <p className="rounded-xl bg-bubble px-3 py-2.5 text-ink-soft text-sm">
              {kind === "memory" ? words.empty.memory(bot.name) : words.empty.instructions}
            </p>
          ) : (
            <ol aria-label={words.list} className="flex flex-col gap-1.5">
              {versions.map((version, index) => (
                <VersionRow
                  key={version.id}
                  version={version}
                  older={versions[index + 1]}
                  newest={index === 0}
                  who={authorOf(version, versions, bot, words)}
                  onRestore={() => setRestoring(version)}
                />
              ))}
            </ol>
          )}
        </div>
      </Dialog>
      {restoring && (
        <Confirm
          title={words.restoreConfirm.title}
          confirmLabel={words.restoreConfirm.confirm}
          onClose={() => setRestoring(null)}
          onConfirm={() =>
            attempt(words.failed.restore, () =>
              api.call("versions.restore", { versionId: restoring.id }),
            )
          }
        >
          {words.restoreConfirm[kind](bot.name, when(restoring.createdAt))}
        </Confirm>
      )}
    </>
  );
}
