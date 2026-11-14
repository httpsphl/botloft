// The rows of "What the agents cost" (spec 8.7): one per agent, model, day
// or routine, from what the daemon sums, with each one's part of the total.

import type { Messages } from "../../i18n";
import { shortDay, tokens, usedTokens } from "../../lib/format";
import { modelName } from "../../lib/models";
import type { BotTokens, CostGroup } from "../../lib/protocol.gen";

export type CostView = "agents" | "models" | "days" | "routines";

export interface CostRow {
  key: string;
  label: string;
  /** Under the name: the crew and how often the agent worked, or the routine's agent. */
  detail: string | null;
  /** The agent's color, for its face. */
  color: string | null;
  archived: boolean;
  /** US dollars at API prices; `null` when the agent's engine gives none. */
  cost: number | null;
  /** The tokens, written for an agent with no price. */
  tokens: string;
}

type Strings = Messages["account"]["usage"]["costs"];

export function agentRows(bots: BotTokens[], u: Strings): CostRow[] {
  return bots.map((bot) => ({
    key: bot.botId,
    label: bot.name,
    // Only a compaction: no turn to count.
    detail: bot.turns > 0 ? u.detail(bot.crew, bot.turns) : bot.crew,
    color: bot.color,
    archived: bot.archived,
    cost: bot.cost,
    tokens: tokens(usedTokens(bot.tokens)),
  }));
}

export function groupRows(view: CostView, groups: CostGroup[], u: Strings): CostRow[] {
  return groups.map((group) => ({
    key: group.key,
    label: groupLabel(view, group, u),
    detail: view === "routines" ? (group.botName ?? null) : null,
    color: view === "routines" ? (group.color ?? null) : null,
    archived: false,
    cost: group.cost,
    tokens: "",
  }));
}

function groupLabel(view: CostView, group: CostGroup, u: Strings): string {
  switch (view) {
    case "models":
      return group.key === "" ? u.noModel : modelName(group.key);
    case "days":
      return shortDay(group.key);
    default:
      return group.name ?? (group.key === "" ? u.notRoutine : u.deletedRoutine);
  }
}

/** What the rows with a price add up to. */
export function totalCost(rows: CostRow[]): number {
  return rows.reduce((sum, row) => sum + (row.cost ?? 0), 0);
}
