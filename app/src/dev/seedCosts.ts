// Dev only: what the preview's agents cost at API prices (spec 8.7), over
// the last few days, on more than one model, and one routine's share.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const HOUR = 60 * 60_000;
const DAY = 24 * HOUR;

export function seedCosts(
  fake: FakeBotloft,
  bots: { scout: BotId; writer: BotId; reviewer: BotId; analyst: BotId },
): void {
  const now = Date.now();
  const routine = [...fake.routines.routines.values()].find((found) => found.botId === bots.scout);
  const part = (
    botId: BotId,
    ago: number,
    cost: number,
    model: string | null,
    fromRoutine = false,
  ) =>
    fake.costParts.push({
      botId,
      at: now - ago,
      cost,
      model,
      routineId: fromRoutine && routine ? routine.id : null,
    });
  part(bots.scout, HOUR, 1.84, "claude-opus-5-5");
  part(bots.scout, 2 * HOUR, 0.12, "claude-haiku-4-5-20251001");
  part(bots.scout, 3 * HOUR, 0.64, "claude-opus-5-5", true);
  part(bots.writer, 30 * 60_000, 0.92, "claude-sonnet-5-5");
  part(bots.reviewer, 4 * HOUR, 0.31, "claude-sonnet-5-5");
  part(bots.analyst, 5 * HOUR, 0.05, "claude-haiku-4-5-20251001");
  part(bots.scout, DAY + HOUR, 2.4, "claude-opus-5-5", true);
  part(bots.writer, 2 * DAY, 1.1, "claude-sonnet-5-5");
  part(bots.reviewer, 3 * DAY, 0.4, null);
}
