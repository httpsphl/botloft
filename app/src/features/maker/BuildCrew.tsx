// One crew of the Maker's plan in its card (spec 36.4): the crew, its chief,
// its agents and what runs on its own, each with a box to leave it out.

import type { ReactNode } from "react";
import { useT } from "../../i18n";
import type { BotModel } from "../../lib/protocol.gen";
import { SelectField, TextArea, TextField } from "../../ui/Field";
import { describeSchedule } from "../routines/describe";
import type { CrewStep, Plan, Step } from "./plan";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

interface Props {
  plan: Plan;
  index: number;
  out: ReadonlySet<number>;
  kept: ReadonlySet<Step>;
  toggle(index: number): void;
  edit(index: number, change: Partial<Step>): void;
}

export function BuildCrew({ plan, index, out, kept, toggle, edit }: Props) {
  const t = useT();
  const c = t.maker.card;
  const crew = plan.steps[index] as CrewStep;
  const included = !out.has(index);
  const mine = plan.steps
    .map((step, at) => ({ step, at }))
    .filter(({ step }) => step.op !== "create_crew" && step.crew === crew.ref);
  return (
    <fieldset className="flex flex-col gap-2 rounded-xl bg-panel px-3 py-2.5">
      <Include label={c.include(crew.name)} checked={included} onChange={() => toggle(index)}>
        <span className="font-semibold">{crew.name}</span>
        <span className="text-muted text-xs">{c.newCrew}</span>
      </Include>
      {included && (
        <div className="flex flex-col gap-2 pl-6">
          {crew.why && <p className="text-ink-soft text-xs">{crew.why}</p>}
          <TextField
            label={t.crews.dialog.name}
            value={crew.name}
            onChange={(event) => edit(index, { name: event.target.value })}
          />
          <details>
            <summary className="cursor-pointer text-muted text-xs hover:text-ink-soft">
              {c.goal} · {c.memory}
            </summary>
            <div className="mt-2 flex flex-col gap-2">
              <TextArea
                label={c.goal}
                hint={c.goalHint}
                rows={3}
                value={crew.goal}
                onChange={(event) => edit(index, { goal: event.target.value })}
              />
              <TextArea
                label={c.memory}
                hint={c.memoryHint}
                rows={3}
                value={crew.memory ?? ""}
                onChange={(event) => edit(index, { memory: event.target.value })}
              />
            </div>
          </details>
          <p className="text-sm">{c.chief(crew.lead_name ?? "")}</p>
          {mine.some(({ step }) => step.op === "add_agent") && (
            <p className="font-medium text-muted text-xs">{c.agents}</p>
          )}
          {mine.map(({ step, at }) =>
            step.op === "add_agent" ? (
              <div key={at} className="flex flex-col gap-1.5">
                <Include
                  label={c.include(step.name)}
                  checked={kept.has(step)}
                  onChange={() => toggle(at)}
                >
                  <span className="font-medium">{step.name}</span>
                  <span className="truncate text-ink-soft text-xs">{step.role}</span>
                </Include>
                {kept.has(step) && (
                  <div className="grid grid-cols-2 gap-2 pl-6">
                    <TextField
                      label={t.chat.suggestion.name}
                      value={step.name}
                      onChange={(event) => edit(at, { name: event.target.value })}
                    />
                    <SelectField
                      label={t.chat.suggestion.model}
                      value={step.model ?? "default"}
                      onChange={(event) => edit(at, { model: event.target.value as BotModel })}
                      options={MODELS.map((value) => ({ value, label: t.chat.model.names[value] }))}
                    />
                  </div>
                )}
              </div>
            ) : null,
          )}
          {mine.some(({ step }) => step.op === "add_routine") && (
            <p className="font-medium text-muted text-xs">{c.routines}</p>
          )}
          {mine.map(({ step, at }) =>
            step.op === "add_routine" ? (
              <Include
                key={at}
                label={c.include(step.name)}
                checked={kept.has(step)}
                onChange={() => toggle(at)}
              >
                <span className="font-medium">{step.name}</span>
                <span className="text-ink-soft text-xs">
                  {describeSchedule(step.schedule, t.routines.when)} ·{" "}
                  {c.forAgent(step.agent === "chief" ? (crew.lead_name ?? "") : step.agent)}
                </span>
              </Include>
            ) : null,
          )}
        </div>
      )}
    </fieldset>
  );
}

export function Include({
  label,
  checked,
  onChange,
  children,
}: {
  label: string;
  checked: boolean;
  onChange(): void;
  children: ReactNode;
}) {
  return (
    <label className="flex min-w-0 cursor-pointer items-center gap-2 text-sm">
      <input
        type="checkbox"
        aria-label={label}
        checked={checked}
        onChange={onChange}
        className="size-4 shrink-0 accent-[var(--accent)]"
      />
      <span className="flex min-w-0 flex-wrap items-baseline gap-x-2">{children}</span>
    </label>
  );
}
