// Changes the Maker's plan makes to crews that already exist (spec 36.8):
// one box per crew, each step with what it is now and what it becomes. The
// owner leaves out any step; archiving and deleting start unchecked.

import type { ReactNode } from "react";
import { useT } from "../../i18n";
import type { BotEffort, BotModel } from "../../lib/protocol.gen";
import { describeSchedule } from "../routines/describe";
import { Include } from "./BuildCrew";
import { onExisting, type Plan, type Step } from "./plan";

interface Props {
  plan: Plan;
  kept: ReadonlySet<Step>;
  toggle(index: number): void;
}

export function BuildChanges({ plan, kept, toggle }: Props) {
  const t = useT();
  const c = t.maker.card;
  return (
    <>
      {onExisting(plan).map(([crew, steps]) => (
        <fieldset
          key={crew}
          aria-label={c.changesIn(crew)}
          className="flex flex-col gap-2 rounded-xl bg-panel px-3 py-2.5"
        >
          <p className="font-semibold text-sm">{c.changesIn(crew)}</p>
          {steps.map(({ step, at }) => (
            <div key={at} className="flex flex-col gap-1 pl-1">
              <Include
                label={c.include(title(step))}
                checked={kept.has(step)}
                onChange={() => toggle(at)}
              >
                <span
                  className={
                    step.op === "archive_agent" || step.op === "delete_routine"
                      ? "font-medium text-danger"
                      : "font-medium"
                  }
                >
                  {title(step)}
                </span>
              </Include>
              <div className="flex flex-col gap-1 pl-6 text-ink-soft text-xs" data-selectable>
                {step.why && <p>{step.why}</p>}
                <Details step={step} />
              </div>
            </div>
          ))}
        </fieldset>
      ))}
    </>
  );

  function title(step: Step): string {
    switch (step.op) {
      case "add_agent":
        return c.addAgent(step.name);
      case "add_routine":
        return c.addRoutine(step.name);
      case "change_agent":
        return c.changeAgent(step.before?.name ?? step.agent);
      case "archive_agent":
        return c.archiveAgent(step.before?.name ?? step.agent);
      case "change_routine":
        return c.changeRoutine(step.before?.name ?? step.routine);
      case "delete_routine":
        return c.deleteRoutine(step.before?.name ?? step.routine);
      default:
        return step.name;
    }
  }
}

function Details({ step }: { step: Step }) {
  const t = useT();
  const c = t.maker.card;
  const model = (value?: BotModel) => (value ? t.chat.model.names[value] : "");
  const effort = (value?: BotEffort) =>
    !value || value === "default" ? c.effortDefault : t.chat.effort.names[value];
  const when = (step: Step) =>
    "schedule" in step && step.schedule ? describeSchedule(step.schedule, t.routines.when) : "";
  switch (step.op) {
    case "add_agent":
      return (
        <>
          <p>{step.role}</p>
          <Folded label={c.newInstructions} text={step.instructions} />
        </>
      );
    case "add_routine":
      return (
        <p>
          {when(step)} · {c.forAgent(step.agent)}
        </p>
      );
    case "archive_agent":
      return (
        <>
          {step.before && <p>{step.before.role}</p>}
          <p>{c.archiveHint}</p>
        </>
      );
    case "delete_routine":
      return step.before ? <p>{describeSchedule(step.before.schedule, t.routines.when)}</p> : null;
    case "change_agent": {
      const before = step.before;
      return (
        <>
          {step.name && <Was label={c.fieldName} before={before?.name} after={step.name} />}
          {step.role && <Was label={c.fieldRole} before={before?.role} after={step.role} />}
          {step.model && (
            <Was label={c.fieldModel} before={model(before?.model)} after={model(step.model)} />
          )}
          {step.effort && (
            <Was
              label={c.fieldEffort}
              before={effort(before?.effort)}
              after={effort(step.effort)}
            />
          )}
          {step.instructions && <Folded label={c.newInstructions} text={step.instructions} />}
          {step.add_instructions && (
            <Folded label={c.addedInstructions} text={step.add_instructions} />
          )}
          {(step.instructions || step.add_instructions) && before && (
            <Folded label={c.oldInstructions} text={before.instructions} />
          )}
        </>
      );
    }
    case "change_routine": {
      const before = step.before;
      return (
        <>
          {step.enabled === false && <p>{c.turnOff}</p>}
          {step.enabled === true && <p>{c.turnOn}</p>}
          {step.name && <Was label={c.fieldName} before={before?.name} after={step.name} />}
          {step.schedule && (
            <Was
              label={c.fieldWhen}
              before={before ? describeSchedule(before.schedule, t.routines.when) : undefined}
              after={when(step)}
            />
          )}
          {step.prompt && <Folded label={c.newText} text={step.prompt} />}
          {step.prompt && before && <Folded label={c.oldText} text={before.prompt} />}
        </>
      );
    }
    default:
      return null;
  }
}

function Was({
  label,
  before,
  after,
}: {
  label: string;
  before?: string | undefined;
  after: string;
}) {
  return (
    <p>
      <span className="text-muted">{label}: </span>
      {before !== undefined && before !== after && <s className="text-muted">{before}</s>}
      {before !== undefined && before !== after && " → "}
      <span className="text-ink">{after}</span>
    </p>
  );
}

function Folded({ label, text }: { label: string; text: string }): ReactNode {
  return (
    <details>
      <summary className="cursor-pointer text-muted hover:text-ink-soft">{label}</summary>
      <p className="mt-1 whitespace-pre-wrap text-ink">{text}</p>
    </details>
  );
}
