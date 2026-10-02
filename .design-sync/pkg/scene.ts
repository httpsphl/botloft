// Builders for the data Botloft's screens show: crews, bots, chat items and
// messages, in the protocol's exact shape (app/src/lib/protocol.gen.ts) and
// with the defaults the app's fake daemon gives new ones. A design passes
// them to BotloftProvider and to the chat components, so it never has to
// spell out every field of a Bot by hand.

import { SITE_TOOL } from "../../app/src/features/browser/SiteCard";
import { PLAN_TOOL } from "../../app/src/features/chat/PlanCard";
import { SUGGEST_TOOL } from "../../app/src/features/chat/SuggestionCard";
import { ROUTINE_TOOL } from "../../app/src/features/routines/RoutineRequestCard";
import {
  AVATAR_PALETTE,
  type Activity,
  type ApprovalItem,
  type ApprovalStatus,
  type Bot,
  type BotModel,
  type BotState,
  type ChatItem,
  type ChatBody,
  type Crew,
  type Message,
  type MessageKind,
  type PermissionMode,
  type Schedule,
  type ToolStatus,
} from "../../app/src/lib/protocol.gen";
import { ago, id } from "./ids";
import { zone } from "./routines";

const slug = (name: string) => name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "bot";

export interface CrewSeed {
  name: string;
  paused?: boolean;
}

export function makeCrew({ name, paused = false }: CrewSeed): Crew {
  const folder = slug(name);
  return {
    id: id("crw"),
    name,
    slug: folder,
    workFolder: `C:\\Users\\owner\\Botloft\\${folder}\\shared`,
    workFolderChosen: false,
    leadBotId: null,
    paused,
    createdAt: ago(60 * 24 * 3),
    archivedAt: null,
  };
}

export interface BotSeed {
  crew: Crew;
  name: string;
  role?: string;
  /** Defaults to the next color of the app's own palette. */
  color?: string;
  state?: BotState;
  paused?: boolean;
  model?: BotModel;
  permissionMode?: PermissionMode;
  /** Makes it the crew's Chief (sets `crew.leadBotId`). */
  chief?: boolean;
  /** What the sidebar shows under its name. */
  activity?: { kind: Activity["kind"]; text: string; tool?: string; minutesAgo?: number };
  /** Marks it unread in the sidebar: it replied after the owner last looked. */
  repliedMinutesAgo?: number;
}

let paletteIndex = 0;

export function makeBot(seed: BotSeed): Bot {
  const handle = slug(seed.name);
  const bot: Bot = {
    id: id("bot"),
    crewId: seed.crew.id,
    name: seed.name,
    handle,
    slug: handle,
    role: seed.role ?? "",
    instructions: "",
    color: seed.color ?? AVATAR_PALETTE[paletteIndex++ % AVATAR_PALETTE.length] ?? "#FF7A59",
    paused: seed.paused ?? false,
    permissionMode: seed.permissionMode ?? "default",
    model: seed.model ?? "default",
    modelInUse: null,
    effort: "default",
    effortDefault: null,
    context: null,
    state: seed.state ?? "idle",
    generation: 1,
    workspace: `C:\\Users\\owner\\Botloft\\${seed.crew.slug}\\${handle}`,
    lastActivity: seed.activity
      ? {
          kind: seed.activity.kind,
          text: seed.activity.text,
          tool: seed.activity.tool ?? null,
          at: ago(seed.activity.minutesAgo ?? 2),
        }
      : null,
    lastReplyAt: seed.repliedMinutesAgo === undefined ? null : ago(seed.repliedMinutesAgo),
    createdAt: ago(60 * 24 * 2),
    archivedAt: null,
  };
  if (seed.chief) {
    seed.crew.leadBotId = bot.id;
  }
  return bot;
}

function item(bot: Bot, body: ChatBody, minutesAgo: number): ChatItem {
  const at = ago(minutesAgo);
  return { id: id("cht"), botId: bot.id, body, createdAt: at, updatedAt: at };
}

function ask(
  bot: Bot,
  toolName: string,
  input: unknown,
  summary: string,
  extra: { status?: ApprovalStatus; explanation?: string; always?: ApprovalItem["always"]; minutesAgo?: number },
): ChatItem {
  return item(
    bot,
    {
      kind: "approval",
      approvalId: id("apr"),
      toolName,
      summary,
      explanation: extra.explanation ?? null,
      input: JSON.stringify(input),
      status: extra.status ?? "pending",
      note: null,
      always: extra.always ?? null,
    },
    extra.minutesAgo ?? 0,
  );
}

/** One turn of a bot is a list of these, in order, passed to BotRun. */
export const chat = {
  reply(bot: Bot, text: string, minutesAgo = 0): ChatItem {
    return item(bot, { kind: "reply", text }, minutesAgo);
  },
  /** A tool call, e.g. `{ name: "WebSearch", summary: "on-device AI news" }`. */
  tool(
    bot: Bot,
    tool: { name: string; summary: string; status?: ToolStatus; output?: string; file?: string },
    minutesAgo = 0,
  ): ChatItem {
    return item(
      bot,
      {
        kind: "tool",
        toolUseId: id("tool"),
        name: tool.name,
        summary: tool.summary,
        explanation: null,
        input: "{}",
        status: tool.status ?? "done",
        output: tool.output ?? null,
        file: tool.file ?? null,
      },
      minutesAgo,
    );
  },
  /** The bot asks to run a command. */
  askCommand(
    bot: Bot,
    command: string,
    extra: { explanation?: string; status?: ApprovalStatus; minutesAgo?: number } = {},
  ): ChatItem {
    return ask(bot, "Bash", { command }, command, {
      ...extra,
      always: { toolName: "Bash", kind: "command", value: command },
    });
  },
  /** The bot asks to use a website it has not used before; the card names its site. */
  askSite(bot: Bot, url: string, extra: { status?: ApprovalStatus; minutesAgo?: number } = {}): ChatItem {
    const site = new URL(url).hostname.replace(/^www\./, "");
    return ask(bot, SITE_TOOL, { site, url }, site, extra);
  },
  /** A Chief suggests a new bot for its crew. */
  suggestBot(
    chief: Bot,
    bot: { name: string; role: string; instructions?: string; model?: BotModel; reason: string },
    extra: { status?: ApprovalStatus; minutesAgo?: number } = {},
  ): ChatItem {
    return ask(chief, SUGGEST_TOOL, { instructions: "", ...bot }, bot.name, extra);
  },
  /** A plan, in markdown, the bot asks to follow. */
  askPlan(bot: Bot, plan: string, extra: { status?: ApprovalStatus; minutesAgo?: number } = {}): ChatItem {
    // As the daemon does: the plan's first line without its heading marks.
    const title = plan.split("\n").map((line) => line.replace(/^#+/, "").trim()).find(Boolean) ?? "";
    return ask(bot, PLAN_TOOL, { plan }, title, extra);
  },
  /** The bot asks to set up a routine (for itself, or for `bot`, another bot's handle). */
  askRoutine(
    bot: Bot,
    routine: { name: string; prompt: string; schedule: Schedule; bot?: string },
    extra: { status?: ApprovalStatus; minutesAgo?: number } = {},
  ): ChatItem {
    return ask(
      bot,
      ROUTINE_TOOL,
      { timezone: zone(), overlap: "skip", missed: "run_once", ...routine },
      routine.name,
      extra,
    );
  },
  /** How the turn ended; closes a BotRun. */
  turn(bot: Bot, seconds: number, minutesAgo = 0): ChatItem {
    return item(bot, { kind: "turn", durationMs: seconds * 1000, tokens: null, error: null }, minutesAgo);
  },
};

export interface MessageSeed {
  /** A bot, or "owner" for you, or "system" for Botloft itself. */
  from: Bot | "owner" | "system";
  to: Bot;
  body: string;
  kind?: MessageKind;
  minutesAgo?: number;
}

export function makeMessage({ from, to, body, kind = "note", minutesAgo = 0 }: MessageSeed): Message {
  const sender = typeof from === "string" ? null : from;
  return {
    id: id("msg"),
    crewId: to.crewId,
    fromKind: sender ? "bot" : (from as "owner" | "system"),
    fromBotId: sender?.id ?? null,
    toBotId: to.id,
    kind,
    body,
    taskId: null,
    routineId: null,
    attachments: [],
    createdAt: ago(minutesAgo),
  };
}

export { AVATAR_PALETTE };
