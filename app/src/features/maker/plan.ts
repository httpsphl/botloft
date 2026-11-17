// A plan of the Maker as its card reads it (spec 36.4): the steps the daemon
// resolved, what the owner leaves out and what they change.

import type { BotEffort, BotModel, Schedule } from "../../lib/protocol.gen";

export const BUILD_TOOL = "mcp__botloft__build";

export interface CrewStep {
  op: "create_crew";
  ref: string;
  name: string;
  goal: string;
  lead_name?: string;
  lead_role?: string;
  lead_model?: BotModel;
  lead_effort?: BotEffort;
  memory?: string;
  work_folder?: string;
  why?: string;
}

export interface AgentStep {
  op: "add_agent";
  crew: string;
  name: string;
  role: string;
  instructions: string;
  model?: BotModel;
  effort?: BotEffort;
  why?: string;
}

export interface RoutineStep {
  op: "add_routine";
  crew: string;
  agent: string;
  name: string;
  prompt: string;
  schedule: Schedule;
  timezone?: string;
  why?: string;
}

/** How a bot of an existing crew is now, filled in by the daemon. */
export interface AgentBefore {
  name: string;
  role: string;
  instructions: string;
  model: BotModel;
  effort: BotEffort;
}

/** How a routine of an existing crew is now, filled in by the daemon. */
export interface RoutineBefore {
  name: string;
  prompt: string;
  schedule: Schedule;
  timezone: string;
  enabled: boolean;
}

export interface ChangeAgentStep {
  op: "change_agent";
  crew: string;
  agent: string;
  name?: string;
  role?: string;
  instructions?: string;
  add_instructions?: string;
  model?: BotModel;
  effort?: BotEffort;
  why?: string;
  before?: AgentBefore;
}

export interface ArchiveAgentStep {
  op: "archive_agent";
  crew: string;
  agent: string;
  why?: string;
  before?: { name: string; role: string };
}

export interface ChangeRoutineStep {
  op: "change_routine";
  crew: string;
  routine: string;
  agent?: string;
  name?: string;
  prompt?: string;
  schedule?: Schedule;
  timezone?: string;
  enabled?: boolean;
  why?: string;
  before?: RoutineBefore;
}

export interface DeleteRoutineStep {
  op: "delete_routine";
  crew: string;
  routine: string;
  agent?: string;
  why?: string;
  before?: RoutineBefore;
}

export type Step =
  | CrewStep
  | AgentStep
  | RoutineStep
  | ChangeAgentStep
  | ArchiveAgentStep
  | ChangeRoutineStep
  | DeleteRoutineStep;

/** Steps that take something away: they start unchecked (spec 36.4). */
export function takesAway(step: Step): boolean {
  return step.op === "archive_agent" || step.op === "delete_routine";
}

/** The steps on crews that already exist, by crew name, in plan order. */
export function onExisting(plan: Plan): [string, { step: Step; at: number }[]][] {
  const refs = new Set(plan.steps.flatMap((step) => (step.op === "create_crew" ? [step.ref] : [])));
  const groups = new Map<string, { step: Step; at: number }[]>();
  plan.steps.forEach((step, at) => {
    if (step.op === "create_crew" || refs.has(step.crew)) {
      return;
    }
    const list = groups.get(step.crew) ?? [];
    list.push({ step, at });
    groups.set(step.crew, list);
  });
  return [...groups.entries()];
}

export interface Plan {
  title: string;
  why: string;
  steps: Step[];
}

/** The plan in a request's input; `null` if it is cut short or not one. */
export function planOf(input: string): Plan | null {
  try {
    const parsed = JSON.parse(input) as Partial<Plan>;
    if (typeof parsed.title !== "string" || !Array.isArray(parsed.steps)) {
      return null;
    }
    return { title: parsed.title, why: parsed.why ?? "", steps: parsed.steps };
  } catch {
    return null;
  }
}

/** A name as the daemon turns it into a handle, to match bots by name. */
function handleOf(name: string): string {
  return name
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/** Whether `routine` runs on `agent` (a bot of the plan). */
export function runsOn(routine: RoutineStep, agent: AgentStep): boolean {
  return routine.crew === agent.crew && handleOf(routine.agent) === handleOf(agent.name);
}

/**
 * The steps left once the owner unchecks some: a crew's bots and routines
 * leave with it, and a bot's routines with the bot.
 */
export function kept(plan: Plan, out: ReadonlySet<number>): Step[] {
  const crewsOut = new Set(
    plan.steps.flatMap((step, index) =>
      step.op === "create_crew" && out.has(index) ? [step.ref] : [],
    ),
  );
  const agentsOut = plan.steps.filter(
    (step, index): step is AgentStep =>
      step.op === "add_agent" && (out.has(index) || crewsOut.has(step.crew)),
  );
  return plan.steps.filter((step, index) => {
    if (out.has(index)) {
      return false;
    }
    if (step.op === "create_crew") {
      return true;
    }
    if (crewsOut.has(step.crew)) {
      return false;
    }
    return step.op !== "add_routine" || !agentsOut.some((agent) => runsOn(step, agent));
  });
}

/** The plan with step `index` changed; a renamed bot keeps its routines. */
export function edited(plan: Plan, index: number, change: Partial<Step>): Plan {
  const before = plan.steps[index];
  const steps = plan.steps.map((step, at) =>
    at === index ? ({ ...step, ...change } as Step) : step,
  );
  if (before?.op === "add_agent" && "name" in change && typeof change.name === "string") {
    const name = change.name;
    return {
      ...plan,
      steps: steps.map((step) =>
        step.op === "add_routine" && runsOn(step, before) ? { ...step, agent: name } : step,
      ),
    };
  }
  return { ...plan, steps };
}

/** How many routines of `steps` run on their own, not by a click. */
export function selfRunning(steps: Step[]): number {
  return steps.filter((step) => step.op === "add_routine" && step.schedule.kind !== "manual")
    .length;
}
