// The fake daemon's Maker (spec 36): its crew and bot, made when a test or
// the preview asks, the plans it builds and `builds.*`.

import type { FakeBotloft, Handlers } from "./fake";
import { botHandlers } from "./fakeBots";
import { crewHandlers } from "./fakeCrews";
import { conflict, notFound } from "./fakeRules";
import type { ApprovalId, Bot, Build, BuildId, Crew, CrewId, Schedule } from "./protocol.gen";

const WEEK_MS = 7 * 24 * 60 * 60 * 1000;

interface Step {
  op: "create_crew" | "add_agent" | "add_routine";
  ref?: string;
  crew?: string;
  name: string;
  goal?: string;
  lead_name?: string;
  lead_role?: string;
  role?: string;
  instructions?: string;
  agent?: string;
  prompt?: string;
  schedule?: Schedule;
  timezone?: string;
}

export class FakeMaker {
  readonly builds: Build[] = [];

  constructor(private readonly fake: FakeBotloft) {}

  /** The Maker's crew and bot, made once, as the daemon makes them at start. */
  ensure(): Bot {
    const have = [...this.fake.crews.values()].find((crew) => crew.system === "maker");
    if (have) {
      return [...this.fake.bots.values()].find((bot) => bot.crewId === have.id) as Bot;
    }
    const crew: Crew = {
      id: this.fake.id("crw"),
      name: "Maker",
      slug: "maker",
      workFolder: "C:\\Users\\owner\\Botloft\\maker\\shared",
      workFolderChosen: false,
      leadBotId: null,
      color: null,
      paused: false,
      autoMode: false,
      system: "maker",
      createdAt: this.fake.now,
      archivedAt: null,
    };
    this.fake.crews.set(crew.id, crew);
    this.fake.changedCrew(crew);
    const bot = botHandlers(this.fake)["bots.create"]({
      crewId: crew.id,
      name: "Maker",
      role: "Builds and tunes your crews",
      instructions: "",
      color: "#FF7A59",
      model: "opus",
    });
    bot.name = "The Maker";
    return bot;
  }

  /** Builds an allowed plan, as the owner left it. */
  build(input: string, approvalId: ApprovalId | null): void {
    const plan = JSON.parse(input) as { title: string; steps: Step[] };
    const crews = new Map<string, CrewId>();
    const bots = new Map<string, Bot>();
    for (const step of plan.steps.filter((each) => each.op === "create_crew")) {
      const crew = crewHandlers(this.fake)["crews.create"]({
        name: step.name,
        lead: {
          name: step.lead_name ?? "Chief",
          role: step.lead_role ?? "Leads the crew",
          instructions: step.goal ?? "",
        },
      });
      crews.set(step.ref ?? "", crew.id);
    }
    for (const step of plan.steps.filter((each) => each.op === "add_agent")) {
      const crewId = crews.get(step.crew ?? "");
      if (crewId) {
        const bot = botHandlers(this.fake)["bots.create"]({
          crewId,
          name: step.name,
          role: step.role ?? "",
          instructions: step.instructions ?? "",
        });
        bots.set(`${step.crew}/${step.name.toLowerCase()}`, bot);
      }
    }
    for (const step of plan.steps.filter((each) => each.op === "add_routine")) {
      const crewId = crews.get(step.crew ?? "");
      const runner =
        bots.get(`${step.crew}/${(step.agent ?? "").toLowerCase()}`)?.id ??
        (crewId ? this.fake.crews.get(crewId)?.leadBotId : null);
      if (runner && step.schedule) {
        this.fake.routines.handlers()["routines.create"]({
          botId: runner,
          name: step.name,
          prompt: step.prompt ?? "",
          schedule: step.schedule,
          timezone: step.timezone ?? "America/Sao_Paulo",
        });
      }
    }
    this.builds.unshift({
      id: this.fake.id("bld") as BuildId,
      botId: this.ensure().id,
      approvalId,
      title: plan.title,
      crewIds: [...crews.values()],
      appliedAt: this.fake.now,
      undoUntil: this.fake.now + WEEK_MS,
      undoneAt: null,
    });
  }

  handlers(): Pick<Handlers, "builds.list" | "builds.undo"> {
    return {
      "builds.list": () => this.builds,
      "builds.undo": ({ buildId }) => {
        const build = this.builds.find((each) => each.id === buildId);
        if (!build) {
          throw notFound(`build ${buildId}`);
        }
        if (build.undoneAt !== null) {
          throw conflict(`build ${buildId} was already undone`);
        }
        for (const crewId of build.crewIds) {
          crewHandlers(this.fake)["crews.archive"]({ crewId });
        }
        build.undoneAt = this.fake.now;
        return build;
      },
    };
  }
}
