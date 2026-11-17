// The Maker's plan (spec 36.4, 36.8): crews, agents and routines to build in
// one go, and changes to the crews that already exist. The owner leaves out or changes any part, then builds it, or says no
// with a note for the Maker. Once answered it shrinks to one line.

import { Ban, Hammer } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard } from "../../ui/ChatCard";
import { useAnswer } from "../chat/useAnswer";
import { BuildChanges } from "./BuildChanges";
import { BuildCrew } from "./BuildCrew";
import { BuildDone } from "./BuildDone";
import { edited, kept, planOf, type Step, selfRunning, takesAway } from "./plan";

export function BuildCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const c = t.maker.card;
  const asked = planOf(approval.input);
  const [plan, setPlan] = useState(asked);
  // Archiving and deleting start unchecked: the owner opts in.
  const [out, setOut] = useState<ReadonlySet<number>>(
    () => new Set(asked?.steps.flatMap((step, index) => (takesAway(step) ? [index] : [])) ?? []),
  );
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: c.buildFailed,
    deny: c.declineFailed,
  });
  if (approval.status !== "pending") {
    return <BuildDone approval={approval} />;
  }
  const steps = plan ? kept(plan, out) : [];
  const left = new Set<Step>(steps);
  const hasCrew = steps.length > 0;
  const changed = out.size > 0 || JSON.stringify(plan) !== JSON.stringify(asked);
  const runs = selfRunning(steps);
  const toggle = (index: number) =>
    setOut((current) => {
      const next = new Set(current);
      if (!next.delete(index)) {
        next.add(index);
      }
      return next;
    });
  const edit = (index: number, change: Partial<Step>) =>
    setPlan((current) => (current ? edited(current, index, change) : current));

  return (
    <SectionCard
      label={c.title(plan?.title ?? approval.summary)}
      icon={Hammer}
      tone="accent"
      footer={
        <>
          {plan && runs > 0 && <p className="mb-2 text-muted text-xs">{c.runs(runs)}</p>}
          {plan && !hasCrew && <p className="mb-2 text-danger text-xs">{c.nothingLeft}</p>}
          <NoteArea
            label={c.noteLabel}
            value={note}
            onChange={setNote}
            placeholder={c.notePlaceholder}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button
              variant="primary"
              icon={Hammer}
              disabled={busy || !plan || !hasCrew}
              onClick={() =>
                plan && answer(true, changed ? JSON.stringify({ ...plan, steps }) : undefined)
              }
            >
              {c.build}
            </Button>
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {c.decline}
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-3 px-4 py-3">
        {plan?.why && (
          <p className="text-ink-soft text-sm" data-selectable>
            <span className="font-medium text-ink">{c.why}: </span>
            {plan.why}
          </p>
        )}
        {plan?.steps.map((step, index) =>
          step.op === "create_crew" ? (
            <BuildCrew
              key={step.ref}
              plan={plan}
              index={index}
              out={out}
              kept={left}
              toggle={toggle}
              edit={edit}
            />
          ) : null,
        )}
        {plan && <BuildChanges plan={plan} kept={left} toggle={toggle} />}
        <p className="sr-only">{bot.name}</p>
      </div>
    </SectionCard>
  );
}
