// An answered plan of the Maker as one line (spec 36.4): built, with Open
// and Undo for a week; or not built.

import { Ban, Check, TimerOff, Undo2 } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { SettledLine } from "../../ui/ChatCard";
import { Confirm } from "../../ui/Confirm";
import { attempt } from "../../ui/toast";
import { onExisting, planOf } from "./plan";
import { useBuilds } from "./useBuilds";

export function BuildDone({ approval }: { approval: ApprovalItem }) {
  const t = useT();
  const c = t.maker.card;
  const title = planOf(approval.input)?.title ?? approval.summary;
  if (approval.status === "denied") {
    return <SettledLine icon={Ban} tone="danger" text={c.declined(title)} note={approval.note} />;
  }
  if (approval.status !== "allowed") {
    return <SettledLine icon={TimerOff} tone="quiet" text={c.expired(title)} />;
  }
  const changes = onExisting(planOf(approval.input) ?? { title, why: "", steps: [] }).length > 0;
  return <Built approvalId={approval.approvalId} title={title} changes={changes} />;
}

function Built({
  approvalId,
  title,
  changes,
}: {
  approvalId: string;
  title: string;
  changes: boolean;
}) {
  const t = useT();
  const c = t.maker.card;
  const api = useApi();
  const { builds, reload } = useBuilds();
  const selectCrew = useApp((state) => state.selectCrew);
  const [asking, setAsking] = useState(false);
  const build = builds.find((each) => each.approvalId === approvalId);
  const undoable = build && build.undoneAt === null && Date.now() < build.undoUntil;
  const first = build?.crewIds[0];
  return (
    <div className="flex flex-wrap items-center gap-2">
      <SettledLine
        icon={build?.undoneAt ? Undo2 : Check}
        tone={build?.undoneAt ? "quiet" : "ok"}
        text={
          build?.undoneAt ? (build.crewIds.length > 0 ? c.undone : c.undoneChanges) : c.built(title)
        }
      />
      {build && !build.undoneAt && first && (
        <Button size="sm" onClick={() => selectCrew(first)}>
          {c.open}
        </Button>
      )}
      {undoable && (
        <Button size="sm" icon={Undo2} onClick={() => setAsking(true)}>
          {c.undo}
        </Button>
      )}
      {asking && build && (
        <Confirm
          title={c.undoTitle}
          confirmLabel={c.undo}
          onClose={() => setAsking(false)}
          onConfirm={async () => {
            await attempt(c.undoFailed, () => api.call("builds.undo", { buildId: build.id }));
            await reload();
          }}
        >
          {build.crewIds.length > 0 && c.undoBody(build.crewIds.length)}
          {changes && ` ${c.undoChanges}`}
        </Confirm>
      )}
    </div>
  );
}
